use std::collections::BTreeSet;

use crate::NFA;

pub struct RegexParser {
    chars: Vec<char>,
    pos: usize,
}

impl RegexParser {
    pub fn parse(regex: &str) -> Result<NFA, String> {
        if regex.is_empty() {
            return Ok(NFA::empty());
        }

        let mut parser = RegexParser {
            chars: regex.chars().collect(),
            pos: 0,
        };

        parser.parse_regex()
    }

    fn parse_regex(&mut self) -> Result<NFA, String> {
        let mut nfa = NFA::empty();

        while self.pos < self.chars.len() {
            match self.chars[self.pos] {
                ')' => return Ok(nfa),
                '(' => {
                    self.pos += 1; // Skip '('
                    let inner = self.parse_regex()?;

                    if self.pos >= self.chars.len() || self.chars[self.pos] != ')' {
                        return Err("Unmatched parenthesis".to_string());
                    }

                    self.pos += 1; // Skip ')'

                    let op = self.parse_operators(inner)?;

                    nfa = if nfa.is_empty() {
                        op
                    } else {
                        NFA::concat(nfa, op)
                    };
                }
                '|' => {
                    self.pos += 1; // Skip '|'
                    let right = self.parse_regex()?;
                    return Ok(NFA::union(nfa, right));
                }
                '[' => {
                    self.pos += 1; // Skip '['
                    let class = self.parse_char_class()?;
                    let op = self.parse_operators(class)?;

                    nfa = if nfa.is_empty() {
                        op
                    } else {
                        NFA::concat(nfa, op)
                    };
                }
                '\\' => {
                    if self.pos + 1 >= self.chars.len() {
                        return Err("Escape at end of regex".to_string());
                    }

                    self.pos += 1; // Skip '\'
                    let char_nfa = NFA::from_char(self.chars[self.pos]);
                    self.pos += 1; // Skip escaped char

                    let op = self.parse_operators(char_nfa)?;

                    nfa = if nfa.is_empty() {
                        op
                    } else {
                        NFA::concat(nfa, op)
                    };
                }
                c => {
                    if "()[].*+?|\\".contains(c) && c != '.' && c != '*' && c != '+' && c != '?' {
                        return Err(format!("Unescaped special character: {}", c));
                    }

                    let char_nfa = if c == '.' {
                        // '.' matches any character except newline
                        let mut chars = BTreeSet::new();
                        for i in 0..=127u8 {
                            let ch = i as char;
                            if ch != '\n' {
                                chars.insert(ch);
                            }
                        }
                        NFA::from_char_class(chars)
                    } else {
                        NFA::from_char(c)
                    };
                    self.pos += 1; // Skip the character

                    let op = self.parse_operators(char_nfa)?;

                    nfa = if nfa.is_empty() {
                        op
                    } else {
                        NFA::concat(nfa, op)
                    };
                }
            }
        }

        Ok(nfa)
    }

    fn parse_operators(&mut self, nfa: NFA) -> Result<NFA, String> {
        if self.pos < self.chars.len() {
            match self.chars[self.pos] {
                '*' => {
                    self.pos += 1;
                    Ok(NFA::kleene(nfa))
                }
                '+' => {
                    self.pos += 1;
                    Ok(NFA::plus(nfa))
                }
                '?' => {
                    self.pos += 1;
                    Ok(NFA::optional(nfa))
                }
                _ => Ok(nfa),
            }
        } else {
            Ok(nfa)
        }
    }

    fn parse_char_class(&mut self) -> Result<NFA, String> {
        let mut class = BTreeSet::new();
        let mut negate = false;

        // Check if the character class is negated [^abc]
        if self.pos < self.chars.len() && self.chars[self.pos] == '^' {
            negate = true;
            self.pos += 1;
        }

        while self.pos < self.chars.len() && self.chars[self.pos] != ']' {
            if self.pos + 2 < self.chars.len()
                && self.chars[self.pos + 1] == '-'
                && self.chars[self.pos + 2] != ']'
            {
                // Handle character range like a-z
                let start = self.chars[self.pos];
                let end = self.chars[self.pos + 2];

                if start > end {
                    return Err(format!("Invalid range: {}-{}", start, end));
                }

                let start_code = start as u32;
                let end_code = end as u32;

                for code in start_code..=end_code {
                    if let Some(c) = char::from_u32(code) {
                        class.insert(c);
                    }
                }

                self.pos += 3; // Skip start, -, end
            } else if self.chars[self.pos] == '\\' && self.pos + 1 < self.chars.len() {
                // Handle escaped characters
                self.pos += 1; // Skip '\'
                class.insert(self.chars[self.pos]);
                self.pos += 1; // Skip escaped char
            } else {
                // Handle single character
                class.insert(self.chars[self.pos]);
                self.pos += 1; // Skip character
            }
        }

        if self.pos >= self.chars.len() {
            return Err("Unterminated character class".to_string());
        }

        // Skip the closing ]
        self.pos += 1;

        let result = if negate {
            // For negated classes, include all ASCII except those in the class
            let mut negated_class = BTreeSet::new();
            for i in 0..128u8 {
                let c = i as char;
                if !class.contains(&c) {
                    negated_class.insert(c);
                }
            }
            NFA::from_char_class(negated_class)
        } else {
            NFA::from_char_class(class)
        };

        Ok(result)
    }
}
