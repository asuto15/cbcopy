use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;


fn read_file(filename: &str) -> String {
    let mut f = File::open(filename).expect("File not found");

    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("Failed to read file");
    return contents;
}

fn get_relative_path(filepath: &str) -> String {
    let absolute_path = Path::new(filepath).canonicalize().expect("Failed to resolve absolute path");
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
}


fn main() {
    let args: Vec<String> = env::args().collect();

    for arg in &args[1..] {
        let filename = get_relative_path(arg);
        let code = read_file(&filename);
        print_code(&filename, code);
    }
}
