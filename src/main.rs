use lex::ParseError;
use lex::NFA;

fn main() -> Result<(), ParseError> {
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

    let nfa = NFA::new(String::from("a"))?;
    println!("{}", nfa);

    let nfa = NFA::new(String::from("ab"))?;
    println!("{}", nfa);

    let nfa = NFA::new(String::from("ab?"))?;
    println!("{}", nfa);

    let nfa = NFA::new(String::from("ab*"))?;
    println!("{}", nfa);

    let nfa = NFA::new(String::from("([a-z]|[0-9])*"))?;
    println!("{}", nfa);

    let nfa = NFA::new(String::from("(((((ab)?)c*)v*)u+)*"))?;
    println!("{}", nfa);

    let nfa = NFA::new(String::from("[\\w]"))?;
    println!("{}", nfa);

    Ok(())
}
