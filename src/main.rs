use lex::LexFile;
use lex::DFA;
use lex::ArgsParser;

fn main() -> Result<(), String> {
    let parser = ArgsParser::new();

    if parser.args().len() < 2 {
        return Err("usage: program [options] file".to_string());
    }
    
    // let language = parser.get_argument("--language", "c");
    // println!("language: {}", language);

    let output = parser.get_argument("-t", "lex.yy.c");
    println!("output: {}", output);
    
    let input = parser.get_file();
    println!("input: {}", input);
    
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
