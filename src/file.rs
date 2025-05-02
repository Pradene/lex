use std::{collections::BTreeMap, fs};

use crate::{NFA, DFA, Regex};

pub enum LexSection {
    Definitions,
    Rules,
    Code,
}

type Definitions = BTreeMap<String, String>;

pub struct Rule {
    pub pattern: String,
    pub nfa: NFA,
    pub action: String,
}

pub struct PendingPattern {
    pub pattern: String,
    pub line_number: usize,
}

pub struct LexFile {
    pub definitions_code: Vec<String>,
    pub definitions: Definitions,
    pub rules: Vec<Rule>,
    pub code: String,
}

impl LexFile {
    pub fn new(path: &str) -> Result<LexFile, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file '{}': {}", path, e))?;
        let lines: Vec<&str> = content.split('\n').collect();

        let mut parser = LexParser::new(path, lines);
        parser.parse()?;

        Ok(LexFile {
            definitions_code: parser.definitions_code,
            definitions: parser.definitions,
            rules: parser.rules,
            code: parser.code,
        })
    }

    pub fn dfa(&self) -> Result<DFA, String> {
        let mut combined_nfa = NFA::empty();
        
        for rule in &self.rules {
            let regex = Regex::new(&rule.pattern)
                .map_err(|e| format!("Invalid pattern '{}': {}", rule.pattern, e))?;
            
            let mut fragment = NFA::from(regex);
            for state in fragment.final_states.clone() {
                fragment.add_action(state, rule.action.clone());
            }
            
            combined_nfa = NFA::union(combined_nfa, fragment);
        }

        Ok(DFA::from(combined_nfa))
    }
}

struct LexParser<'a> {
    path: &'a str,
    lines: Vec<&'a str>,
    definitions_code: Vec<String>,
    definitions: Definitions,
    rules: Vec<Rule>,
    code: String,
    pending_patterns: Vec<PendingPattern>,
    current_section: LexSection,
    line_index: usize,
}

impl<'a> LexParser<'a> {
    fn new(path: &'a str, lines: Vec<&'a str>) -> Self {
        Self {
            path,
            lines,
            definitions_code: Vec::new(),
            definitions: BTreeMap::new(),
            rules: Vec::new(),
            code: String::new(),
            pending_patterns: Vec::new(),
            current_section: LexSection::Definitions,
            line_index: 0,
        }
    }

    fn parse(&mut self) -> Result<(), String> {
        while self.line_index < self.lines.len() {
            let line = self.lines[self.line_index].trim();
            let line_number = self.line_index + 1;

            if line == "%%" {
                self.handle_section_separator()?;
                self.line_index += 1;
                continue;
            }

            if self.should_skip_line(line) {
                self.line_index += 1;
                continue;
            }

            match self.current_section {
                LexSection::Definitions => self.process_definitions_line(line, line_number)?,
                LexSection::Rules => self.process_rules_line(line, line_number)?,
                LexSection::Code => self.process_code_line(),
            }

            self.line_index += 1;
        }

        self.validate_final_state()
    }

    fn handle_section_separator(&mut self) -> Result<(), String> {
        match self.current_section {
            LexSection::Definitions => self.current_section = LexSection::Rules,
            LexSection::Rules => self.current_section = LexSection::Code,
            LexSection::Code => return Err(format!(
                "Unexpected section separator at line {}",
                self.line_index + 1
            )),
        }
        Ok(())
    }

    fn should_skip_line(&self, line: &str) -> bool {
        line.is_empty() || line.starts_with("//") || line.starts_with('#')
    }

    fn process_definitions_line(&mut self, line: &str, line_number: usize) -> Result<(), String> {
        if line.starts_with("%{") {
            self.process_definitions_code_block()
        } else {
            self.process_definition(line, line_number)
        }
    }

    fn process_definitions_code_block(&mut self) -> Result<(), String> {
        self.line_index += 1; // Skip opening %{
        
        while self.line_index < self.lines.len() {
            let line = self.lines[self.line_index];
            if line.trim().starts_with("%}") {
                self.line_index += 1;
                return Ok(());
            }
            self.definitions_code.push(line.to_string());
            self.line_index += 1;
        }

        Err(format!("{}: Unclosed definitions code block", self.path))
    }

    fn process_definition(&mut self, line: &str, line_number: usize) -> Result<(), String> {
        let (name, value) = line.split_once(' ')
            .ok_or_else(|| format!("{}:{}: Invalid definition format", self.path, line_number))?;

        let expanded_value = self.expand_macros(value.trim())?;
        self.definitions.insert(name.trim().to_string(), expanded_value);
        Ok(())
    }

    fn process_rules_line(&mut self, line: &str, line_number: usize) -> Result<(), String> {
        let (pattern, action) = Self::split_pattern_action(line)
            .map_err(|e| format!("{}:{}: {}", self.path, line_number, e))?;

        let expanded_pattern = self.expand_macros(&pattern)?;
        self.handle_rule_action(expanded_pattern, action, line_number)
    }

    fn handle_rule_action(
        &mut self,
        pattern: String,
        action: String,
        line_number: usize,
    ) -> Result<(), String> {
        if action == "|" {
            self.pending_patterns.push(PendingPattern { pattern, line_number });
            return Ok(());
        }

        if action.starts_with('{') {
            self.process_action_block(pattern, action, line_number)
        } else {
            self.commit_rule(pattern, action, line_number)
        }
    }

    fn process_action_block(
        &mut self,
        pattern: String,
        mut action: String,
        line_number: usize,
    ) -> Result<(), String> {
        let mut brace_count = action.chars().filter(|c| *c == '{').count() as i32;
        brace_count -= action.chars().filter(|c| *c == '}').count() as i32;

        self.pending_patterns.push(PendingPattern { pattern, line_number });
        let mut current_line = self.line_index;

        while brace_count > 0 && current_line < self.lines.len() - 1 {
            current_line += 1;
            let line = self.lines[current_line].trim();
            action.push('\n');
            action.push_str(line);

            brace_count += line.chars().filter(|c| *c == '{').count() as i32;
            brace_count -= line.chars().filter(|c| *c == '}').count() as i32;
        }

        if brace_count != 0 {
            return Err(format!("{}: Unclosed action block starting at line {}", self.path, line_number));
        }

        self.line_index = current_line;
        self.commit_pending_rules(action)
    }

    fn commit_pending_rules(&mut self, action: String) -> Result<(), String> {
        for pending in self.pending_patterns.drain(..) {
            self.rules.push(Rule::new(pending.pattern, action.clone())?);
        }
        Ok(())
    }

    fn commit_rule(&mut self, pattern: String, action: String, _line_number: usize) -> Result<(), String> {
        if !self.pending_patterns.is_empty() {
            self.commit_pending_rules(action.clone())?;
        }
        self.rules.push(Rule::new(pattern, action)?);
        Ok(())
    }

    fn process_code_line(&mut self) {
        self.code.push_str(self.lines[self.line_index]);
        self.code.push('\n');
    }

    fn expand_macros(&self, input: &str) -> Result<String, String> {
        let mut result = input.to_string();
        let mut changed = true;

        while changed {
            changed = false;
            for (name, value) in &self.definitions {
                let macro_ref = format!("{{{}}}", name);
                if result.contains(&macro_ref) {
                    result = result.replace(&macro_ref, value);
                    changed = true;
                }
            }
        }

        Ok(result)
    }

    fn validate_final_state(&self) -> Result<(), String> {
        if !self.pending_patterns.is_empty() {
            let first_pending = &self.pending_patterns[0];
            Err(format!(
                "{}:{}: Pattern without action",
                self.path, first_pending.line_number
            ))
        } else {
            Ok(())
        }
    }

    fn split_pattern_action(line: &str) -> Result<(String, String), String> {
        PatternParser::new().parse(line)
    }
}

struct PatternParser {
    in_bracket: i32,
    in_quote: bool,
    escaped: bool,
    split_pos: Option<usize>,
}

impl PatternParser {
    fn new() -> Self {
        Self {
            in_bracket: 0,
            in_quote: false,
            escaped: false,
            split_pos: None,
        }
    }

    fn parse(mut self, line: &str) -> Result<(String, String), String> {
        for (i, c) in line.char_indices() {
            if self.handle_escape(c) { continue; }
            
            match c {
                '[' if !self.in_quote => self.in_bracket += 1,
                ']' if !self.in_quote => self.in_bracket = (self.in_bracket - 1).max(0),
                '"' => self.in_quote = !self.in_quote,
                ' ' | '\t' if self.should_split() => {
                    self.split_pos = Some(i);
                    break;
                }
                _ => {}
            }
        }

        self.split_result(line)
    }

    fn handle_escape(&mut self, c: char) -> bool {
        if c == '\\' && !self.escaped {
            self.escaped = true;
            true
        } else {
            self.escaped = false;
            false
        }
    }

    fn should_split(&self) -> bool {
        self.in_bracket == 0 && !self.in_quote && !self.escaped
    }

    fn split_result(self, line: &str) -> Result<(String, String), String> {
        match self.split_pos {
            Some(pos) => {
                let pattern = line[..pos].trim();
                let action = line[pos..].trim();
                
                if pattern.is_empty() {
                    Err("Empty pattern in rule".into())
                } else {
                    Ok((pattern.to_string(), action.to_string()))
                }
            }
            None => Err(format!("Could not split rule and action: {}", line)),
        }
    }
}

impl Rule {
    pub fn new(pattern: String, action: String) -> Result<Rule, String> {
        let nfa = NFA::new(&pattern)
            .map_err(|e| format!("Invalid regex pattern '{}': {}", pattern, e))?;
        Ok(Rule { pattern, nfa, action })
    }
}