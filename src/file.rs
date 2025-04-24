use std::{collections::BTreeMap, fs};

pub enum LexSection {
    Definitions,
    Rules,
    Code,
}

type Definition = BTreeMap<String, String>;

pub struct Rule {
    pub pattern: String,
    pub action: String,
}

pub struct PendingPattern {
    pub pattern: String,
    pub line_number: usize,
}

pub struct LexFile {
    pub definitions: Definition,
    pub rules: Vec<Rule>,
    pub code: String,
}

impl LexFile {
    pub fn new(path: &String) -> Result<LexFile, String> {
        let content =
            fs::read_to_string(path.clone()).map_err(|e| format!("Failed to read file: {}", e))?;
        let lines: Vec<&str> = content.split('\n').collect();

        let mut definitions: Definition = BTreeMap::new();
        let mut rules: Vec<Rule> = Vec::new();
        let mut code = String::new();
        let mut pending_patterns: Vec<PendingPattern> = Vec::new();

        let mut current_section = LexSection::Definitions;
        let mut action_accumulator = String::new();
        let mut in_action_block = false;
        let mut brace_count = 0;

        let mut i = 0;
        while i < lines.len() {
            let line_number = i + 1;
            let line = lines[i].trim();

            if line == "%%" {
                match current_section {
                    LexSection::Definitions => current_section = LexSection::Rules,
                    LexSection::Rules => current_section = LexSection::Code,
                    _ => {
                        return Err(format!(
                            "Unexpected section separator at line {}",
                            line_number
                        ))
                    }
                }

                i += 1;

                continue;
            }

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with("//") || line.starts_with("#") {
                i += 1;

                continue;
            }

            match current_section {
                LexSection::Definitions => {
                    // Handle definitions section - parse name/regex or C code
                    if line.starts_with("%{") {
                        // code block in definitions section
                        i += 1;
                        while i < lines.len() && !lines[i].trim().starts_with("%}") {
                            // Add to definitions as special C code block
                            i += 1;
                        }

                        if i < lines.len() {
                            i += 1; // Skip the closing %}
                        }
                    } else if let Some(pos) = line.find(' ') {
                        let name = line[..pos].trim().to_string();
                        let value = line[pos..].trim().to_string();

                        // Expand any macros in the definition value
                        let expanded_value = LexFile::expand_macros(&value, &definitions)?;

                        // Add to both our definition collection and our lookup map
                        definitions.insert(name, expanded_value);

                        i += 1;
                    } else {
                        i += 1;
                    }
                }

                LexSection::Rules => {
                    // In rules section
                    if in_action_block {
                        // We're inside a multi-line action block
                        action_accumulator.push_str(line);
                        action_accumulator.push('\n');

                        // Count braces to track nested blocks
                        for c in line.chars() {
                            if c == '{' {
                                brace_count += 1;
                            } else if c == '}' {
                                brace_count -= 1;
                                if brace_count == 0 {
                                    in_action_block = false;

                                    // Add rules for all pending patterns
                                    if !pending_patterns.is_empty() {
                                        for pending_pattern in &pending_patterns {
                                            rules.push(Rule {
                                                pattern: pending_pattern.pattern.clone(),
                                                action: action_accumulator.clone(),
                                            });
                                        }
                                        pending_patterns.clear();
                                    }
                                    action_accumulator.clear();
                                    break;
                                }
                            }
                        }

                        i += 1;
                    } else {
                        // Parse pattern/action rule
                        if let Some(pos) = line.find(|c: char| c.is_whitespace()) {
                            let pattern = line[..pos].trim().to_string();
                            let action_start = line[pos..].trim();

                            let pattern = LexFile::expand_macros(&pattern, &definitions)?;

                            if action_start == "|" {
                                // OR pattern - add to pending patterns
                                pending_patterns.push(PendingPattern {
                                    pattern,
                                    line_number,
                                });
                                i += 1;
                            } else if action_start.starts_with("{") {
                                // Start of action block
                                action_accumulator = action_start.to_string();
                                action_accumulator.push('\n');

                                // Count opening braces
                                brace_count = 1;
                                for c in action_start.chars().skip(1) {
                                    if c == '{' {
                                        brace_count += 1;
                                    } else if c == '}' {
                                        brace_count -= 1;
                                    }
                                }

                                if brace_count > 0 {
                                    in_action_block = true;

                                    // Add this pattern to pending patterns if it's not already
                                    // part of an OR group
                                    if pending_patterns.is_empty() {
                                        pending_patterns.push(PendingPattern {
                                            pattern,
                                            line_number,
                                        });
                                    }
                                } else {
                                    // Single-line action
                                    if !pending_patterns.is_empty() {
                                        for pending_pattern in &pending_patterns {
                                            rules.push(Rule {
                                                pattern: pending_pattern.pattern.clone(),
                                                action: action_accumulator.clone(),
                                            });
                                        }
                                        pending_patterns.clear();
                                    } else {
                                        rules.push(Rule {
                                            pattern,
                                            action: action_accumulator.clone(),
                                        });
                                    }
                                    action_accumulator.clear();
                                }
                                i += 1;
                            } else {
                                // Single-line action without braces
                                let action = action_start.to_string();

                                if !pending_patterns.is_empty() {
                                    for pending_pattern in &pending_patterns {
                                        rules.push(Rule {
                                            pattern: pending_pattern.pattern.clone(),
                                            action: action.clone(),
                                        });
                                    }
                                    pending_patterns.clear();
                                } 
                                rules.push(Rule { pattern, action });
                                
                                i += 1;
                            }
                        } else {
                            // Malformed line
                            return Err(format!(
                                "Error: {}:{} - Malformed rule",
                                path, line_number
                            ));
                        }
                    }
                }

                LexSection::Code => {
                    i += 1;
                    while i < lines.len() {
                        code.push_str(lines[i]);
                        code.push('\n');
                        i += 1;
                    }

                    break;
                }
            }
        }

        // Check for pending patterns without actions
        if !pending_patterns.is_empty() {
            let pattern = pending_patterns.get(0).unwrap();
            return Err(format!(
                "Error: {}:{} - Pattern without action",
                path, pattern.line_number
            ));
        }

        // Check for unclosed action blocks
        if in_action_block {
            return Err(format!("Error: {} - Unclosed action block", path));
        }

        Ok(LexFile {
            definitions,
            rules,
            code,
        })
    }

    fn expand_macros(
        pattern: &str,
        definitions: &BTreeMap<String, String>,
    ) -> Result<String, String> {
        let mut result = pattern.to_string();
        let mut changed = true;
        let max_iterations = 100;
        let mut iteration = 0;

        while changed && iteration < max_iterations {
            changed = false;
            iteration += 1;

            for (name, value) in definitions {
                let macro_ref = format!("{{{}}}", name);
                if result.contains(&macro_ref) {
                    result = result.replace(&macro_ref, value);
                    changed = true;
                }
            }
        }

        if iteration >= max_iterations {
            return Err(format!(
                "Potential circular reference in macro definitions for pattern: {}",
                pattern
            ));
        }

        Ok(result)
    }
}

impl Rule {
    pub fn new(pattern: String, action: String) -> Rule {
        Rule { pattern, action }
    }
}
