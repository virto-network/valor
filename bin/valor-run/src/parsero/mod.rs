// use clap::Parser;
use std::path::Path;

/// Simple module to have cli parser options
#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about)]
#[command(next_line_help = true)]
pub struct Args {
    /// List of modules
    /// The program can be called with several parameters
    // Like this: -p path1 -p path2
    #[arg(short = 'p', long = "plugin_path")]
    pub plugin_path: Vec<String>,

    /// X
    #[arg(short = 'x', long = "xxx")]
    // To make a parameter optional just declare it as Option
    pub x: Option<u8>,

    /// Y
    #[arg(short = 'y', long = "yyy", default_value_t = 1)]
    // Without Option is mandatory a value for the parameter, but with
    // default_value_t it's possible to assign directly a value.
    pub y: u8,

    /// All Active
    #[arg(short = 'a', long = "all_active", default_value_t = false)]
    // Without Option is mandatory a value for the parameter, but with
    // default_value_t it's possible to assign directly a value.
    pub all_active: bool,

    /// Quiet
    #[arg(short = 'q', long = "quiet", default_value_t = false)]
    pub quiet: bool,
}

impl Args {
    pub fn check_plugin_paths(&self) -> bool {
        let mut files_exist = true;
        for path in &self.plugin_path {
            let file_to_check = Path::new(&path);
            if !file_to_check.exists() {
                files_exist = false;
                println!("The file {path} doesn't exist, please check the path!");
            }
        }
        files_exist
    }
}
