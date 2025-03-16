use std::collections::BTreeSet;
use std::fmt;

pub enum Regex {
    Char(char),
    Union(Box<Regex>, Box<Regex>),
    Concat(Box<Regex>, Box<Regex>),
    Option(Box<Regex>),
    Plus(Box<Regex>),
    Kleene(Box<Regex>),
    Bounded(Box<Regex>, usize, Option<usize>),
    CharClass(BTreeSet<char>),
    NegatedCharClass(BTreeSet<char>),
    Dot,
    StartAnchor, // ^ at start of regex
    EndAnchor,   // $ at end of regex
    Empty,       // Represents empty/epsilon regex
}

impl fmt::Display for Regex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt(f, 0)
    }
}

impl Regex {
    pub fn new(regex: &str) -> Result<Regex, String> {
        let mut parser = RegexParser::new(regex);
        parser.parse()
    }

    fn fmt(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        // Helper to create indentation
        let indent_str = " ".repeat(indent);

        match self {
            Regex::Char(c) => {
                write!(f, "{}Char('{}')", indent_str, c)?;
            }
            Regex::CharClass(chars) => {
                write!(f, "{}CharClass[", indent_str)?;
                for c in chars {
                    write!(f, "{}", c)?;
                }
                write!(f, "]")?;
            }
            Regex::NegatedCharClass(chars) => {
                write!(f, "{}NegatedCharClass[^", indent_str)?;
                for c in chars {
                    write!(f, "{}", c)?;
                }
                write!(f, "]")?;
            }
            Regex::Dot => {
                write!(f, "{}Dot", indent_str)?;
            }
            Regex::StartAnchor => {
                write!(f, "{}StartAnchor", indent_str)?;
            }
            Regex::EndAnchor => {
                write!(f, "{}EndAnchor", indent_str)?;
            }
            Regex::Empty => {
                write!(f, "{}Empty", indent_str)?;
            }
            Regex::Bounded(inner, min, max) => {
                let range = match max {
                    Some(max) if min == max => format!("{}", min),
                    Some(max) => format!("{}-{}", min, max),
                    None => format!("{}+", min),
                };
                writeln!(f, "{}Bounded({}) {{", indent_str, range)?;
                inner.fmt(f, indent + 2)?;
                write!(f, "\n{}}}", indent_str)?;
            }
            Regex::Plus(inner) => {
                writeln!(f, "{}Plus {{", indent_str)?;
                inner.fmt(f, indent + 2)?;
                write!(f, "\n{}}}", indent_str)?;
            }
            Regex::Kleene(inner) => {
                writeln!(f, "{}Kleene {{", indent_str)?;
                inner.fmt(f, indent + 2)?;
                write!(f, "\n{}}}", indent_str)?;
            }
            Regex::Option(inner) => {
                writeln!(f, "{}Option {{", indent_str)?;
                inner.fmt(f, indent + 2)?;
                write!(f, "\n{}}}", indent_str)?;
            }
            Regex::Union(left, right) => {
                writeln!(f, "{}Union {{", indent_str)?;
                left.fmt(f, indent + 2)?;
                writeln!(f, ",")?;
                right.fmt(f, indent + 2)?;
                write!(f, "\n{}}}", indent_str)?;
            }
            Regex::Concat(left, right) => {
                writeln!(f, "{}Concat {{", indent_str)?;
                left.fmt(f, indent + 2)?;
                writeln!(f, ",")?;
                right.fmt(f, indent + 2)?;
                write!(f, "\n{}}}", indent_str)?;
            }
        }
        Ok(())
    }
}

pub struct RegexParser {
    chars: Vec<char>,
    pos: usize,
}

impl RegexParser {
    fn new(regex: &str) -> RegexParser {
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

    fn match_string(&mut self, s: &str) -> bool {
        for (i, expected) in s.chars().enumerate() {
            if self.peek(i) != Some(expected) {
                return false;
            }
        }

        for _ in 0..s.len() {
            self.advance();
        }

        true
    }

    pub fn parse(&mut self) -> Result<Regex, String> {
        if self.at_end() {
            return Ok(Regex::Empty);
        }

        let start_anchored = if self.current_char() == Some('^') {
            self.advance();
            true
        } else {
            false
        };

        let mut expr = self.parse_union()?;

        if self.current_char() == Some('$') {
            self.advance();
            expr = Regex::Concat(Box::new(expr), Box::new(Regex::EndAnchor));
        }

        if start_anchored {
            expr = Regex::Concat(Box::new(Regex::StartAnchor), Box::new(expr));
        }

        if !self.at_end() {
            return Err(format!(
                "Unexpected char '{}' at {}",
                self.current_char().unwrap(),
                self.pos
            ));
        }
        Ok(expr)
    }

    fn parse_union(&mut self) -> Result<Regex, String> {
        let mut left = self.parse_concat()?;
        while self.current_char() == Some('|') {
            self.advance();
            let right = self.parse_concat()?;
            left = Regex::Union(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_concat(&mut self) -> Result<Regex, String> {
        let mut factors = Vec::new();
        while let Some(c) = self.current_char() {
            if c == ')' || c == '|' || c == '$' {
                break;
            }
            factors.push(self.parse_operator()?);
        }

        if factors.is_empty() {
            Ok(Regex::Empty)
        } else {
            let mut expr = factors.remove(0);
            for factor in factors {
                expr = Regex::Concat(Box::new(expr), Box::new(factor));
            }
            Ok(expr)
        }
    }

    fn parse_operator(&mut self) -> Result<Regex, String> {
        let mut expr = self.parse_base()?;

        if let Some(c) = self.current_char() {
            match c {
                '*' => {
                    self.advance();
                    expr = Regex::Kleene(Box::new(expr));
                }
                '+' => {
                    self.advance();
                    expr = Regex::Plus(Box::new(expr));
                }
                '?' => {
                    self.advance();
                    expr = Regex::Option(Box::new(expr));
                }
                '{' => {
                    expr = self.parse_bounded(expr)?;
                }
                _ => {}
            }
        }

        Ok(expr)
    }

    fn parse_bounded(&mut self, expr: Regex) -> Result<Regex, String> {
        if self.current_char() != Some('{') {
            return Ok(expr);
        }
        self.advance();

        let min = self.parse_number()?;
        let mut max = None;

        if self.current_char() == Some(',') {
            self.advance();
            if self.current_char() != Some('}') {
                max = Some(self.parse_number()?);
            }
        } else {
            max = Some(min);
        }

        if self.current_char() != Some('}') {
            return Err(format!("Expected '}}' at position {}", self.pos));
        }
        self.advance();

        Ok(Regex::Bounded(Box::new(expr), min, max))
    }

    fn parse_number(&mut self) -> Result<usize, String> {
        let mut num = String::new();
        while let Some(c) = self.current_char() {
            if c.is_ascii_digit() {
                num.push(c);
                self.advance();
            } else {
                break;
            }
        }
        num.parse().map_err(|_| "Invalid number".to_string())
    }

    fn parse_base(&mut self) -> Result<Regex, String> {
        match self.current_char() {
            Some('(') => self.parse_group(),
            Some('[') => self.parse_char_class(),
            Some('.') => {
                self.advance();
                Ok(Regex::Dot)
            }
            Some('\\') => self.parse_escape(),
            Some(c) => {
                self.advance();
                if c == '$' || c == '^' {
                    return Err(format!("Unexpected '{}' in middle of pattern", c));
                }
                Ok(Regex::Char(c))
            }
            None => Err("Unexpected end of pattern".to_string()),
        }
    }

    fn parse_group(&mut self) -> Result<Regex, String> {
        self.advance(); // Skip '('
        let _ = self.check_non_capturing_group(); // Ignore non-capturing for AST
        let expr = self.parse_union()?;
        if self.current_char() != Some(')') {
            return Err("Unmatched parenthesis".to_string());
        }
        self.advance(); // Skip ')'
        Ok(expr)
    }

    fn check_non_capturing_group(&mut self) -> bool {
        if self.current_char() == Some('?') && self.peek(1) == Some(':') {
            self.pos += 2;
            true
        } else {
            false
        }
    }

    fn parse_char_class(&mut self) -> Result<Regex, String> {
        self.advance(); // Skip '['
        let mut chars = BTreeSet::new();
        let mut negated = false;

        if self.current_char() == Some('^') {
            negated = true;
            self.advance();
        }

        // Handle POSIX character classes first
        if self.current_char() == Some('[') && self.peek(1) == Some(':') {
            // Parse POSIX character class like [[:alpha:]] directly
            return self.parse_posix_class(negated);
        }

        while let Some(c) = self.current_char() {
            if c == ']' {
                break;
            }

            // Handle POSIX class notation inside character class
            if c == '[' && self.peek(1) == Some(':') {
                let class_chars = self.parse_named_class()?;
                for char in class_chars {
                    chars.insert(char);
                }
                continue;
            }

            if c == '\\' {
                self.advance();
                self.parse_escape_in_class(&mut chars)?;
            } else if let Some('-') = self.peek(1) {
                if let Some(end) = self.peek(2) {
                    if end != ']' {
                        let start = c;
                        self.advance(); // Skip start
                        self.advance(); // Skip '-'
                        let end = self.consume_char().unwrap();
                        self.add_char_range(start, end, &mut chars)?;
                        continue;
                    }
                }
                chars.insert(c);
                self.advance();
            } else {
                chars.insert(c);
                self.advance();
            }
        }

        if self.current_char() != Some(']') {
            return Err("Unclosed character class".to_string());
        }
        self.advance();

        Ok(if negated {
            Regex::NegatedCharClass(chars)
        } else {
            Regex::CharClass(chars)
        })
    }

    fn parse_posix_class(&mut self, negated: bool) -> Result<Regex, String> {
        // We're at the first '[' of something like [[:alpha:]]
        self.advance(); // Skip first '['

        if !self.match_string(":") {
            return Err("Expected ':' after '[' in POSIX class".to_string());
        }

        let mut class_name = String::new();
        while let Some(c) = self.current_char() {
            if c == ':' {
                break;
            }
            class_name.push(c);
            self.advance();
        }

        if !self.match_string(":]") {
            return Err("Expected ':]' at end of POSIX class".to_string());
        }

        if self.current_char() != Some(']') {
            return Err("Expected ']' to close character class".to_string());
        }
        self.advance(); // Skip final ']'

        // Look up the named class
        let class_name_str = class_name.as_str();
        if let Some(chars) = self.get_named_class(class_name_str) {
            Ok(if negated {
                Regex::NegatedCharClass(chars.clone())
            } else {
                Regex::CharClass(chars.clone())
            })
        } else {
            Err(format!("Unknown POSIX character class '{}'", class_name))
        }
    }

    fn get_named_class(&self, name: &str) -> Option<BTreeSet<char>> {
        match name {
            "alpha" => Some(('a'..='z').chain('A'..='Z').collect()),
            "digit" => Some(('0'..='9').collect()),
            "alnum" => Some(('a'..='z').chain('A'..='Z').chain('0'..='9').collect()),
            "space" => Some(
                [' ', '\t', '\n', '\r', '\u{000B}', '\u{000C}']
                    .iter()
                    .cloned()
                    .collect(),
            ),
            "punct" => Some("!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~".chars().collect()),
            "graph" => Some((0x21..=0x7E).filter_map(|c| char::from_u32(c)).collect()),
            "print" => Some((0x20..=0x7E).filter_map(|c| char::from_u32(c)).collect()),
            "xdigit" => Some(('0'..='9').chain('a'..='f').chain('A'..='F').collect()),
            "blank" => Some([' ', '\t'].iter().cloned().collect()),
            "cntrl" => Some(
                (0x00..=0x1F)
                    .chain(0x7F..=0x7F)
                    .filter_map(|c| char::from_u32(c))
                    .collect(),
            ),
            "lower" => Some(('a'..='z').collect()),
            "upper" => Some(('A'..='Z').collect()),
            _ => None,
        }
    }

    fn parse_named_class(&mut self) -> Result<BTreeSet<char>, String> {
        // We're at the first '[' of something like [:alpha:]
        self.advance(); // Skip '['

        if !self.match_string(":") {
            return Err("Expected ':' after '[' in named class".to_string());
        }

        let mut class_name = String::new();
        while let Some(c) = self.current_char() {
            if c == ':' {
                break;
            }
            class_name.push(c);
            self.advance();
        }

        if !self.match_string(":]") {
            return Err("Expected ':]' at end of named class".to_string());
        }

        // Look up the named class
        let class_name_str = class_name.as_str();
        if let Some(chars) = self.get_named_class(class_name_str) {
            Ok(chars.clone())
        } else {
            Err(format!("Unknown named character class '{}'", class_name))
        }
    }

    fn add_char_range(
        &self,
        start: char,
        end: char,
        chars: &mut BTreeSet<char>,
    ) -> Result<(), String> {
        if start > end {
            return Err("Invalid character range".to_string());
        }
        for c in start..=end {
            chars.insert(c);
        }
        Ok(())
    }

    fn parse_escape_in_class(&mut self, chars: &mut BTreeSet<char>) -> Result<(), String> {
        match self.current_char() {
            Some('d') => {
                ('0'..='9').for_each(|c| {
                    chars.insert(c);
                });
                self.advance();
            }
            Some('w') => {
                ('a'..='z').chain('A'..='Z').chain('0'..='9').for_each(|c| {
                    chars.insert(c);
                });
                chars.insert('_');
                self.advance();
            }
            Some('s') => {
                [' ', '\t', '\n', '\r', '\u{000B}', '\u{000C}']
                    .iter()
                    .for_each(|&c| {
                        chars.insert(c);
                    });
                self.advance();
            }
            // Handle lex-specific escape sequences
            Some('a') => {
                chars.insert('\u{0007}'); // Bell
                self.advance();
            }
            Some('b') => {
                chars.insert('\u{0008}'); // Backspace
                self.advance();
            }
            Some('f') => {
                chars.insert('\u{000C}'); // Form feed
                self.advance();
            }
            Some('n') => {
                chars.insert('\n');
                self.advance();
            }
            Some('r') => {
                chars.insert('\r');
                self.advance();
            }
            Some('t') => {
                chars.insert('\t');
                self.advance();
            }
            Some('v') => {
                chars.insert('\u{000B}'); // Vertical tab
                self.advance();
            }
            Some(c) => {
                chars.insert(c);
                self.advance();
            }
            None => return Err("Escape at end of pattern".to_string()),
        }
        Ok(())
    }

    fn parse_escape(&mut self) -> Result<Regex, String> {
        self.advance(); // Skip '\'
        match self.current_char() {
            Some('d') => {
                self.advance();
                Ok(Regex::CharClass(('0'..='9').collect()))
            }
            Some('D') => {
                self.advance();
                let mut set: BTreeSet<char> = (0..=127).filter_map(|c| char::from_u32(c)).collect();
                set.retain(|c| !c.is_ascii_digit());
                Ok(Regex::NegatedCharClass(set))
            }
            Some('w') => {
                self.advance();
                let mut set: BTreeSet<char> =
                    ('a'..='z').chain('A'..='Z').chain('0'..='9').collect();
                set.insert('_');
                Ok(Regex::CharClass(set))
            }
            Some('W') => {
                self.advance();
                let mut set: BTreeSet<char> = (0..=127).filter_map(|c| char::from_u32(c)).collect();
                set.retain(|c| !c.is_alphanumeric() && *c != '_');
                Ok(Regex::NegatedCharClass(set))
            }
            Some('s') => {
                self.advance();
                Ok(Regex::CharClass(
                    [' ', '\t', '\n', '\r', '\u{000B}', '\u{000C}']
                        .iter()
                        .cloned()
                        .collect(),
                ))
            }
            Some('S') => {
                self.advance();
                let mut set: BTreeSet<char> = (0..=127).filter_map(|c| char::from_u32(c)).collect();
                set.retain(|c| ![' ', '\t', '\n', '\r', '\u{000B}', '\u{000C}'].contains(c));
                Ok(Regex::NegatedCharClass(set))
            }
            // Handle lex-specific escape sequences
            Some('a') => {
                self.advance();
                Ok(Regex::Char('\u{0007}')) // Bell
            }
            Some('b') => {
                self.advance();
                Ok(Regex::Char('\u{0008}')) // Backspace
            }
            Some('f') => {
                self.advance();
                Ok(Regex::Char('\u{000C}')) // Form feed
            }
            Some('n') => {
                self.advance();
                Ok(Regex::Char('\n'))
            }
            Some('r') => {
                self.advance();
                Ok(Regex::Char('\r'))
            }
            Some('t') => {
                self.advance();
                Ok(Regex::Char('\t'))
            }
            Some('v') => {
                self.advance();
                Ok(Regex::Char('\u{000B}')) // Vertical tab
            }
            // Handle octal escapes \123
            Some(c) if c.is_digit(8) => {
                let mut octal = String::new();
                octal.push(c);
                self.advance();

                // Read up to 2 more octal digits
                for _ in 0..2 {
                    if let Some(digit) = self.current_char() {
                        if digit.is_digit(8) {
                            octal.push(digit);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }

                // Parse octal value
                let value = u32::from_str_radix(&octal, 8)
                    .map_err(|_| "Invalid octal escape".to_string())?;

                if let Some(c) = char::from_u32(value) {
                    Ok(Regex::Char(c))
                } else {
                    Err("Invalid character code".to_string())
                }
            }
            // Handle hex escapes \xHH
            Some('x') => {
                self.advance();
                let mut hex = String::new();

                // Read exactly 2 hex digits
                for _ in 0..2 {
                    if let Some(digit) = self.current_char() {
                        if digit.is_digit(16) {
                            hex.push(digit);
                            self.advance();
                        } else {
                            return Err("Expected hex digit in \\x escape".to_string());
                        }
                    } else {
                        return Err("Incomplete hex escape".to_string());
                    }
                }

                // Parse hex value
                let value =
                    u32::from_str_radix(&hex, 16).map_err(|_| "Invalid hex escape".to_string())?;

                if let Some(c) = char::from_u32(value) {
                    Ok(Regex::Char(c))
                } else {
                    Err("Invalid character code".to_string())
                }
            }
            Some(c) => {
                self.advance();
                Ok(Regex::Char(c))
            }
            None => Err("Escape at end of pattern".to_string()),
        }
    }
}
