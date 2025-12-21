use std::{
    fs::File,
    io::{stdout, Write},
};

use lex::ArgsParser;
use lex::CodeGenerator;
use lex::LexFile;

fn main() -> Result<(), String> {
    let parser = ArgsParser::new();

    if parser.args().len() < 2 {
        return Err("usage: program [options] file".to_string());
    }

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
    let dfa = file.dfa()?;

    let generator = CodeGenerator::new(file, dfa);
    let code = generator.code();

    writeln!(output, "{}", code).map_err(|e| format!("{}", e))?;

    Ok(())
}
