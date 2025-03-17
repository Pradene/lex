use lex::LexFile;
use lex::DFA;

use std::env;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    let language = args
        .windows(2)
        .find(|window| window[0] == "--language")
        .map(|window| window[1].clone())
        .unwrap_or_else(|| "c".to_string());
    println!("language: {}", language);

    let input = args
        .windows(2)
        .find(|window| window[0] == "--input")
        .map(|window| window[1].clone())
        .unwrap_or_else(|| panic!("You must provide an input input"));
    println!("input: {}", input);

    let output = args
        .windows(2)
        .find(|window| window[0] == "--output")
        .map(|window| window[1].clone())
        .unwrap_or_else(|| "lex.yy.c".to_string());
    println!("output: {}", output);

    let file = LexFile::new(input)?;

    let dfa = DFA::new(&file)?;

    let tests = vec![String::from("42+1337+(21*19)\n")];
    for test in &tests {
        let actions = dfa.simulate(test);

        for action in actions {
            println!("{}", action.1);
        }
    }

    Ok(())
}
