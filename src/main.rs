use std::fs;

pub struct Rule {
    pub pattern: String,
    pub action: String,
}

fn main() {
    let path = "syntax/scanner.l";

    let content = fs::read_to_string(path).expect("Should have been able to read the file");
    let lines: Vec<&str> = content.split('\n').collect();

    let mut rules: Vec<Rule> = Vec::new();
    let mut current_patterns: Vec<String> = Vec::new();

    for (number, line) in lines.iter().enumerate() {
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
                    current_patterns.push(pattern);
                }

                _ => {
                    if !current_patterns.is_empty() {
                        for pattern in &current_patterns {
                            rules.push(Rule {
                                pattern: pattern.clone(),
                                action: action.clone(),
                            })
                        }

                        current_patterns.clear()
                    }
                    
                    rules.push(Rule { pattern, action });
                }
            }
        } else {
            eprintln!("{}:{}", path, number);
        }
    }

    for rule in rules {
        println!("pattern: {}", rule.pattern);
        println!("action: {}", rule.action);
    }
}
