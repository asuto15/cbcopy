use std::env;
use std::fs::File;
use std::io::prelude::*;


fn read_file(filename: &str) -> String {
    let mut f = File::open(filename).expect("File not found");

    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("Failed to read file");
    return contents;
}

fn print_code(filename: &str, code: String) {
    println!("```");
    println!("// {}", filename);
    println!("{}", code);
    println!("```");
}


fn main() {
    let args: Vec<String> = env::args().collect();

    for arg in &args[1..] {
        let filename = arg;
        let code = read_file(&filename);
        print_code(&filename, code);
    }
}
