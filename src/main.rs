use std::fs::File;
use std::io::Write;
use std::io::stdout;

use lex::CodeGenerator;
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

    let mut output: Box<dyn Write> = if !parser.has_flag("-t") {
        let filename = "lex.yy.c";
        let file = File::create(filename);
        match file {
            Ok(file) => Box::new(file),
            Err(e) => return Err(format!("{e}")),
        }
    } else {
        Box::new(stdout())
    };
    
    let input = parser.get_file();

    let file = LexFile::new(&input)?;
    let dfa = DFA::new(&file)?;

    let generator = CodeGenerator::new(file, dfa);
    let code = generator.generate_code();

    writeln!(output, "{}", code).map_err(|e| format!("{}", e))?;

    Ok(())
}
