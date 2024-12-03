use std::{
    env,
    fs::File,
    io::prelude::*,
    path::Path,
};
use glob::glob;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    patterns: Vec<String>,
}

fn read_file(path: &Path) -> String {
    let mut f = File::open(path).expect("File not found");

    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("Failed to read file");
    contents
}

fn get_relative_path(path: &Path) -> String {
    let absolute_path = path.canonicalize().expect("Failed to resolve absolute path");
    let current_dir = env::current_dir().expect("Failed to get current directory");
    absolute_path.strip_prefix(&current_dir)
        .unwrap_or(&absolute_path)
        .to_string_lossy()
        .to_string()
}

fn print_code(relative_path: &str, code: String) {
    println!("```");
    println!("// {}", relative_path);
    println!("{}", code);
    println!("```");
    println!();
}


fn main() {
    let args = Args::parse();

    let patterns = args.patterns;

    for pattern in patterns {
        match glob(&pattern) {
            Ok(paths) => {
                for result in paths {
                    match result {
                        Ok(path) => {
                            if path.is_file() {
                                let relative_path = get_relative_path(&path);
                                let code = read_file(&path);
                                print_code(&relative_path, code);
                            }
                        }
                        Err(e) => eprintln!("Error matching pattern '{}': {}", pattern, e),
                    }
                }
            }
            Err(e) => eprintln!("Invalid glob pattern '{}': {}", pattern, e),
        }
    }
}
