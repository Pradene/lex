use std::{
    fs::File,
    io::{stdout, Write},
};

use lex::{
    CodeGenerator, LexFile, {App, Opt},
};

fn main() -> Result<(), String> {
    let opts = vec![
        Opt::new("t").help("Redirect stdout to a file".to_string()),
        Opt::new("f").help("Minimize transition table".to_string()),
    ];

    let app = App::new(opts);
    let args = app.parse();

    let mut output: Box<dyn Write> = if args.contains("t") {
        let filename = "lex.yy.c";
        let file = File::create(filename).map_err(|e| format!("{e}"))?;
        Box::new(file)
    } else {
        Box::new(stdout())
    };

    let input = args
        .positional
        .first()
        .ok_or("./lex [options] input_file")?;

    let file = LexFile::new(input)?;
    let dfa = file.dfa()?;
    let generator = CodeGenerator::new(file, dfa);
    let code = generator.code();
    writeln!(output, "{}", code).map_err(|e| format!("{}", e))?;
    Ok(())
}
