use std::collections::HashSet;

pub struct Opt {
    pub name: String,
    pub help: Option<String>,
    pub required: bool,
}

impl Opt {
    pub fn new(name: String) -> Self {
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
    pub executable: String,
    pub flags: HashSet<String>,
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

    pub fn parse_args(&self) -> Args {
        let args: Vec<String> = std::env::args().collect();
        let executable = args[0].clone();
        let mut flags = HashSet::new();
        let mut positional = Vec::new();

        for arg in &args[1..] {
            // Check if it is an option
            if arg.starts_with('-') {
                let flag_name = arg.trim_start_matches('-');
                if let Some(opt) = self.opts.iter().find(|o| o.name == flag_name) {
                    flags.insert(opt.name.clone());
                }
            } else {
                positional.push(arg.clone());
            }
        }

        Args {
            executable,
            flags,
            positional,
        }
    }

    pub fn help(&self) {
        todo!()
    }
}
