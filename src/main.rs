use std::{
    fs::File,
    io::{stdout, Write},
};

use lex::{
    CodeGenerator, LexFile, {App, Opt},
};

fn main() -> Result<(), String> {
    let opts = vec![
        Opt::new("t".to_string()).help("Redirect stdout to a file".to_string()),
        Opt::new("f".to_string()).help("Minimize transition table".to_string()),
    ];

    let app = App::new(opts);
    let args = app.parse_args();

    let mut output: Box<dyn Write> = if args.contains("t") {
        Box::new(stdout())
    } else {
        let filename = "lex.yy.c";
        let file = File::create(filename).map_err(|e| format!("{e}"))?;
        Box::new(file)
    };

    let input = args
        .positional
        .first()
        .ok_or(format!("Usage: {} [options] file", args.executable))?;

    let file = LexFile::new(input)?;
    let dfa = if args.contains("f") {
        file.dfa()?
    } else {
        file.dfa()?.minimize()
    };

    let generator = CodeGenerator::new(file, dfa);
    let code = generator.code();

    writeln!(output, "{}", code).map_err(|e| format!("{}", e))?;
    Ok(())
}
