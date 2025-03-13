use lex::NFA;

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

    // println!("{}", language);

    let expressions = vec![
        String::from("a"),
        String::from("ab"),
        String::from("a|b"),
        String::from("[0-9]"),
        String::from("[a-zA-Z]"),
        String::from("a+"),
        String::from("(a*b)*"),
        String::from("a?"),
        String::from("[0-9]+"),
        String::from("a{4}"),
        String::from("a{0,2}"),
        String::from("a{10,}"),
        String::from("a{0,100000}"),
        // String::from("(a{0,100000}){0,10000}"),
    ];

    for expr in expressions {
        let nfa = NFA::new(expr).unwrap();
        println!("{}", nfa);
    }

    Ok(())
}
