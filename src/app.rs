use std::collections::HashSet;

pub struct Opt {
    pub name: &'static str,
    pub help: Option<String>,
    pub required: bool,
}

impl Opt {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            help: None,
            required: false,
        }
    }

    pub fn help(mut self, help: String) -> Self {
        self.help = Some(help);
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}

pub struct App {
    opts: Vec<Opt>,
}

pub struct Args {
    pub flags: HashSet<&'static str>,
    pub positional: Vec<String>,
}

impl Args {
    pub fn contains(&self, flag: &str) -> bool {
        self.flags.contains(flag)
    }
}

impl App {
    pub fn new(opts: Vec<Opt>) -> Self {
        Self { opts }
    }

    pub fn parse(&self) -> Args {
        let args: Vec<String> = std::env::args().collect();
        let mut flags = HashSet::new();
        let mut positional = Vec::new();

        for arg in &args[1..] {
            // Check if it is an option
            if arg.starts_with('-') {
                let flag_name = arg.trim_start_matches('-');
                if let Some(opt) = self.opts.iter().find(|o| o.name == flag_name) {
                    flags.insert(opt.name);
                }
            } else {
                positional.push(arg.clone());
            }
        }

        Args { flags, positional }
    }

    pub fn help(&self) {
        todo!()
    }
}
