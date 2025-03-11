use lex::nfa::NFA;

fn main() {
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

    // println!("{}", language);

    // Ok(())

    let nfa1 = NFA::from_char('h');
    println!("{}", nfa1);
    let nfa2 = NFA::from_char('a');
    println!("{}", nfa2);

    let nfa = NFA::alternate(nfa1, nfa2);
    println!("{}", nfa);
    let star = NFA::kleene(nfa);

    println!("{}", star);
}
