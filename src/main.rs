use std::fs;
use std::process;

pub struct Rule {
    pub pattern: String,
    pub action: String,
}

pub struct PendingPattern {
    pub pattern: String,
    pub line_number: usize,
}

fn main() {
    let path = "syntax/scanner.l";

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
                            rules.push(Rule {
                                pattern: pending_pattern.pattern.clone(),
                                action: action.clone(),
                            })
                        }

                        pending_patterns.clear()
                    }

                    rules.push(Rule { pattern, action });
                }
            }
        } else {
            eprintln!("Error: {}:{}", path, line_number);
        }
    }

    if !pending_patterns.is_empty() {
        for pending_pattern in &pending_patterns {
            eprintln!("Error: {}:{}", path, pending_pattern.line_number);
        }

        process::exit(1);
    }

    for rule in rules {
        println!("pattern: {}", rule.pattern);
        println!("action: {}", rule.action);
    }
}
