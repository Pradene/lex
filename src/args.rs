use std::env;

pub struct ArgsParser {
    args: Vec<String>
}

impl ArgsParser {
    pub fn new() -> Self {
        let args = env::args().collect();

        Self { args }
    }

    // Parse arugment in a window of size 2 
    // Format:
    //   0: option
    //   1: value
    pub fn get_argument(&self, name: &str, default: &str) -> String {
        self.args
            .windows(2)
            .find(|window| window[0] == name.to_string())
            .map(|window| window[1].clone())
            .unwrap_or_else(|| default.to_string())
    }

    // Get the file path (last argument)
    pub fn get_file(&self) -> String {
        self.args.last().unwrap().clone()
    }

    pub fn args(&self) -> &Vec<String> {
        &self.args
    }
}
