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
    println!("output: {}\n", output);
    
    let input = parser.get_file();
    println!("input: {}\n", input);
    
    let file = LexFile::new(input)?;

    let dfa = DFA::new(&file)?;

    let tests = vec![String::from("42+1337+(21*19)\n")];
    for test in &tests {
        let actions = dfa.simulate(test);

        for action in actions {
            let value = action.0.replace("\n", "\\n");
            let action = action.1;
            println!("{:<12}{}", value, action);
        }
    }

    Ok(())
}
