use lex::{DFA, NFA};

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

    // let em = NFA::empty();
    // println!("{}", em);

    // let nfa1 = NFA::from_char('h');
    // println!("{}", nfa1);
    // let nfa2 = NFA::from_char('a');
    // println!("{}", nfa2);

    // let un = NFA::union(nfa1.clone(), nfa2.clone());
    // println!("{}", un);

    // let star = NFA::kleene(un.clone());
    // println!("{}", star);

    // let con = NFA::concat(nfa1.clone(), nfa2.clone());
    // println!("{}", con);

    // let con_plus = NFA::plus(con.clone());
    // println!("{}", con_plus);

    // let con_opt = NFA::optional(con.clone());
    // println!("{}", con_opt);

    // let dfa = DFA::from(star.clone());
    // println!("{}", dfa);

    let n1 = NFA::from_char('1');
    let n2 = NFA::from_char('2');
    let n3 = NFA::concat(n1, n2);
    println!("{}", n3);

    let a = NFA::from_char('a');
    let b = NFA::from_char('b');

    let c = NFA::union(a.clone(), b.clone());
    let d = NFA::kleene(c);
    println!("{}", d);
    let e = NFA::concat(d, a.clone());
    let f = NFA::concat(e, b.clone());
    let g = NFA::concat(f, b.clone());
    println!("{}", g);

    let dfa = DFA::from(g);
    println!("{}", dfa);
}
