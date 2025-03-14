use lex::Table;
use lex::DFA;
use lex::NFA;

use std::env;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    let _language = args
        .windows(2)
        .find(|window| window[0] == "--language")
        .map(|window| window[1].clone())
        .unwrap_or_else(|| "c".to_string());

    let path = "syntax/scanner.l";
    let table = Table::new(path)?;

    let mut nfa = NFA::empty();

    for rule in table.rules {
        let regex = NFA::new(rule.pattern, rule.action)?;
        nfa = NFA::union(nfa, regex);
    }

    println!("{}", nfa);

    let dfa = DFA::from(nfa);

    let tests = vec![
        String::from(""),
        String::from("0"),
        String::from("bonjour0"),
        String::from("bonjour"),
        String::from("int"),
        String::from("cha_"),
    ];

    for test in &tests {
        let action = dfa.simulate(test);

        match action {
            Some(action) => println!("matched: {}", action),
            None => println!("not matched"),
        }
    }

    Ok(())
}
