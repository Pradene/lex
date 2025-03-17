use lex::Table;
use lex::DFA;
use lex::NFA;

use std::env;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    let language = args
        .windows(2)
        .find(|window| window[0] == "--language")
        .map(|window| window[1].clone())
        .unwrap_or_else(|| "c".to_string());

    println!("language: {}", language);

    let path = args
        .windows(2)
        .find(|window| window[0] == "--input")
        .map(|window| window[1].clone())
        .unwrap_or_else(|| panic!("You must provide an input path"));

    println!("path: {}", path);
    let table = Table::new(path)?;

    let mut nfa = NFA::empty();

    for rule in table.rules {
        let regex = NFA::new(rule.pattern, rule.action)?;
        nfa = NFA::union(nfa, regex);
    }

    let dfa = DFA::from(nfa);

    let tests = vec![String::from("42+1337+(21*19)\n")];

    for test in &tests {
        let actions = dfa.simulate(test);

        for action in actions {
            println!("{}", action.1);
        }
    }

    Ok(())
}
