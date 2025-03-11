use std::fs;

pub struct Rule {
    pub pattern: String,
    pub action: String,
}

pub struct PendingPattern {
    pub pattern: String,
    pub line_number: usize,
}

pub struct Table {
    pub rules: Vec<Rule>,
}

impl Table {
    pub fn new(path: &str) -> Result<Table, String> {
        let content = fs::read_to_string(path).expect("Should have been able to read the file");
        let lines: Vec<&str> = content.split('\n').collect();

        let mut rules: Vec<Rule> = Vec::new();
        let mut pending_patterns: Vec<PendingPattern> = Vec::new();

        for (index, line) in lines.iter().enumerate() {
            let line_number = index + 1;

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() == 2 {
                let pattern = parts[0].trim().to_string();
                let action = parts[1].trim().to_string();

                match action.as_str() {
                    "|" => {
                        pending_patterns.push(PendingPattern {
                            pattern,
                            line_number,
                        });
                    }

                    _ => {
                        if !pending_patterns.is_empty() {
                            for pending_pattern in &pending_patterns {
                                let pattern = &pending_pattern.pattern;
                                rules.push(Rule {
                                    pattern: pattern.clone(),
                                    action: action.clone(),
                                })
                            }

                            pending_patterns.clear()
                        }

                        rules.push(Rule::new(pattern, action));
                    }
                }
            } else {
                eprintln!("Error: {}:{}", path, line_number);
            }
        }

        if !pending_patterns.is_empty() {
            let pattern = pending_patterns.get(0).unwrap();
            return Err(format!("Error: {}:{}", path, pattern.line_number));
        }

        Ok(Table { rules })
    }
}

impl Rule {
    pub fn new(pattern: String, action: String) -> Rule {
        Rule { pattern, action }
    }
}
