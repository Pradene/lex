use lex::NFA;
use lex::DFA;

fn main() -> Result<(), String> {
    // let args: Vec<String> = env::args().collect();
    // let default_lang = "c";

    // let language = args
    //     .windows(2)
    //     .find(|window| window[0] == "--language")
    //     .map(|window| window[1].clone())
    //     .unwrap_or_else(|| default_lang.to_string());

    // let path = "syntax/scanner.l";
    // let table = Table::new(path)?;

    // for rule in table.rules {
    //     println!("{} - {}", rule.pattern, rule.action);
    // }

    let nfa_return = NFA::new(String::from("return"))?;
    let nfa_int = NFA::new(String::from("int"))?;
    let nfa_char = NFA::new(String::from("char"))?;
    let nfa_while = NFA::new(String::from("while"))?;

    let nfa = NFA::union_multiples(vec![
        nfa_return,
        nfa_int,
        nfa_char,
        nfa_while,
    ]);

    let dfa = DFA::from(nfa);
    println!("{}", dfa);

    println!("{}", dfa.simulate("int"));
    println!("{}", dfa.simulate("return"));
    println!("{}", dfa.simulate("cha"));
    println!("{}", dfa.simulate("char"));

    Ok(())
}
