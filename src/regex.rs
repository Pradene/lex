use std::collections::BTreeSet;
use std::fmt;

use crate::NFA;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEnd,
    UnmatchedParenthesis { pos: usize },
    UnterminatedClass,
    InvalidRange { start: char, end: char },
    InvalidEscape { pos: usize },
    InvalidRepetition { min: usize, max: Option<usize> },
    UnexpectedChar { pos: usize, char: char },
    TrailingBackslash,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedEnd => write!(f, "Unexpected end of pattern"),
            ParseError::UnmatchedParenthesis { pos } => {
                write!(f, "Unmatched '(' at position {}", pos)
            }
            ParseError::UnterminatedClass => write!(f, "Unterminated character class"),
            ParseError::InvalidRange { start, end } => {
                write!(f, "Invalid range: {}-{}", start, end)
            }
            ParseError::InvalidEscape { pos } => {
                write!(f, "Invalid escape sequence at position {}", pos)
            }
            ParseError::InvalidRepetition { min, max } => match max {
                Some(max) => write!(f, "Invalid repetition: {{{},{}}}", min, max),
                None => write!(f, "Invalid repetition: {{{}}}", min),
            },
            ParseError::UnexpectedChar { pos, char } => {
                write!(f, "Unexpected character '{}' at position {}", char, pos)
            }
            ParseError::TrailingBackslash => write!(f, "Pattern ends with trailing backslash"),
        }
    }
}

pub struct RegexParser {
    chars: Vec<char>,
    pos: usize,
}

impl RegexParser {
    pub fn new(regex: &str) -> RegexParser {
        RegexParser {
            chars: regex.chars().collect(),
            pos: 0,
        }
    }

    fn current_char(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn consume_char(&mut self) -> Option<char> {
        let c = self.current_char();
        if c.is_some() {
            self.advance();
        }

        c
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn at_end(&self) -> bool {
        self.pos >= self.chars.len()
    }

    pub fn parse(&mut self) -> Result<NFA, ParseError> {
        let nfa = self.parse_union()?;
        if !self.at_end() {
            return Err(ParseError::UnexpectedChar {
                pos: self.pos,
                char: self.current_char().unwrap(),
            });
        }

        Ok(nfa)
    }

    fn parse_union(&mut self) -> Result<NFA, ParseError> {
        let mut nfa = self.parse_concat()?;
        while self.current_char() == Some('|') {
            self.advance(); // skip '|'
            let right = self.parse_concat()?;
            nfa = NFA::union(nfa, right);
        }

        Ok(nfa)
    }

    fn parse_concat(&mut self) -> Result<NFA, ParseError> {
        let mut factors = Vec::new();
        while let Some(c) = self.current_char() {
            if c == ')' || c == '|' {
                break;
            }
            factors.push(self.parse_operator()?);
        }

        Ok(NFA::concat_multiples(factors))
    }

    fn parse_operator(&mut self) -> Result<NFA, ParseError> {
        let mut nfa = self.parse_base()?;

        if let Some(c) = self.current_char() {
            match c {
                '*' => {
                    self.advance();
                    nfa = NFA::kleene(nfa);
                }
                '+' => {
                    self.advance();
                    nfa = NFA::plus(nfa);
                }
                '?' => {
                    self.advance();
                    nfa = NFA::optional(nfa);
                }
                _ => {}
            }
        }

        Ok(nfa)
    }

    fn parse_base(&mut self) -> Result<NFA, ParseError> {
        match self.current_char() {
            Some('(') => self.parse_group(),
            Some('[') => self.parse_char_class(),
            Some('.') => {
                self.advance();
                Ok(NFA::dot())
            }
            Some('\\') => self.parse_escape(),
            Some(c) => {
                self.advance();
                Ok(NFA::char(c))
            }
            None => Err(ParseError::UnexpectedEnd),
        }
    }

    fn parse_group(&mut self) -> Result<NFA, ParseError> {
        let start_pos = self.pos;
        self.advance(); // skip '('

        let _ = self.check_non_capturing_group();

        let nfa = self.parse_union()?;

        if self.current_char() != Some(')') {
            return Err(ParseError::UnmatchedParenthesis { pos: start_pos });
        }

        self.advance(); // skip ')'

        Ok(nfa)
    }

    fn check_non_capturing_group(&mut self) -> bool {
        if self.current_char() == Some('?') && self.peek(1) == Some(':') {
            self.pos += 2; // Skip '?:'
            true
        } else {
            false
        }
    }

    fn parse_char_class(&mut self) -> Result<NFA, ParseError> {
        let mut class = BTreeSet::new();
        let mut negate = false;

        self.advance(); // skip '['

        if self.current_char() == Some('^') {
            negate = true;
            self.advance();
        }

        while let Some(c) = self.current_char() {
            if c == ']' {
                break;
            }

            if self.peek(1) == Some('-') && self.peek(2).is_some() && self.peek(2) != Some(']') {
                let start = c;
                self.pos += 2; // Skip start and '-'
                let end = self.consume_char().unwrap();

                if start > end {
                    return Err(ParseError::InvalidRange { start, end });
                }

                self.add_char_range(start, end, &mut class)?;
            } else if c == '\\' {
                self.advance(); // Skip '\'
                if self.at_end() {
                    return Err(ParseError::TrailingBackslash);
                }

                self.parse_escape_in_class(&mut class)?;
            } else {
                class.insert(c);
                self.advance();
            }
        }

        if self.current_char() != Some(']') {
            return Err(ParseError::UnterminatedClass);
        }

        self.advance(); // Skip ']'

        Ok(if negate {
            NFA::char_class_negated(class)
        } else {
            NFA::char_class(class)
        })
    }

    fn add_char_range(
        &self,
        start: char,
        end: char,
        class: &mut BTreeSet<char>,
    ) -> Result<(), ParseError> {
        let start_code = start as u32;
        let end_code = end as u32;

        for code in start_code..=end_code {
            if let Some(c) = char::from_u32(code) {
                class.insert(c);
            }
        }

        Ok(())
    }

    fn parse_escape_in_class(&mut self, class: &mut BTreeSet<char>) -> Result<(), ParseError> {
        match self.current_char() {
            Some('d') => {
                self.advance();
                for c in '0'..='9' {
                    class.insert(c);
                }
            }
            Some('w') => {
                self.advance();
                for c in 'a'..='z' {
                    class.insert(c);
                }
                for c in 'A'..='Z' {
                    class.insert(c);
                }
                for c in '0'..='9' {
                    class.insert(c);
                }
                class.insert('_');
            }
            Some('s') => {
                self.advance();
                for c in [' ', '\t', '\n', '\r'] {
                    class.insert(c);
                }
            }
            Some(c) => {
                self.advance();
                class.insert(c);
            }
            None => return Err(ParseError::TrailingBackslash),
        }

        Ok(())
    }

    fn parse_escape(&mut self) -> Result<NFA, ParseError> {
        self.advance(); // skip '\\'
        if self.at_end() {
            return Err(ParseError::TrailingBackslash);
        }

        match self.current_char() {
            Some('d') => {
                self.advance();
                let mut digits = BTreeSet::new();
                for c in '0'..='9' {
                    digits.insert(c);
                }
                Ok(NFA::char_class(digits))
            }
            Some('D') => {
                self.advance();
                let mut non_digits = BTreeSet::new();
                for c in (0..=127).map(|i| i as u8 as char) {
                    if !('0'..='9').contains(&c) {
                        non_digits.insert(c);
                    }
                }
                Ok(NFA::char_class(non_digits))
            }
            Some('w') => {
                self.advance();
                let mut word_chars = BTreeSet::new();
                for c in 'a'..='z' {
                    word_chars.insert(c);
                }
                for c in 'A'..='Z' {
                    word_chars.insert(c);
                }
                for c in '0'..='9' {
                    word_chars.insert(c);
                }
                word_chars.insert('_');
                Ok(NFA::char_class(word_chars))
            }
            Some('W') => {
                self.advance();
                let mut non_word_chars = BTreeSet::new();
                for c in (0..=127).map(|i| i as u8 as char) {
                    if !('a'..='z').contains(&c)
                        && !('A'..='Z').contains(&c)
                        && !('0'..='9').contains(&c)
                        && c != '_'
                    {
                        non_word_chars.insert(c);
                    }
                }
                Ok(NFA::char_class(non_word_chars))
            }
            Some('s') => {
                self.advance();
                let mut whitespace = BTreeSet::new();
                for c in [' ', '\t', '\n', '\r'] {
                    whitespace.insert(c);
                }
                Ok(NFA::char_class(whitespace))
            }
            Some('S') => {
                self.advance();
                let mut non_whitespace = BTreeSet::new();
                for c in (0..=127).map(|i| i as u8 as char) {
                    if ![' ', '\t', '\n', '\r'].contains(&c) {
                        non_whitespace.insert(c);
                    }
                }
                Ok(NFA::char_class(non_whitespace))
            }
            Some(c) => {
                self.advance();
                Ok(NFA::char(c))
            }
            None => Err(ParseError::TrailingBackslash),
        }
    }
}
