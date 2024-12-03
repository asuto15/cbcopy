use std::{
    env,
    fs::File,
    io::prelude::*,
    path::Path,
};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    file_paths: Vec<String>,

    #[arg(short, long, default_value = "false")]
    absolute: bool,
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

fn get_absolute_path(path: &Path) -> String {
    path.canonicalize().expect("Failed to resolve absolute path")
        .to_string_lossy()
        .to_string()
}

fn print_code(relative_path: &str, code: String) {
    println!("```");
    println!("// {}", relative_path);
    print!("{}", code);
    println!("```");
    println!();
}


fn main() {
    let args = Args::parse();

    let is_absolute = args.absolute;
    let mut found_any_file = false;
    let file_names = args.file_paths;

    for file_name in &file_names {
        let path = Path::new(&file_name);

        if path.is_file() {
            found_any_file = true;
            let path_to_display = match is_absolute {
                true => {
                    get_absolute_path(&path)
                },
                false => {
                    get_relative_path(&path)
                }
            };
            let code = read_file(&path);
            print_code(&path_to_display, code);
        } else if path.is_dir() {
            eprintln!("{} is a directory", file_name);
        } else {
            eprintln!("{} is not a file", file_name);
        }
    }

    if !found_any_file {
        eprintln!("Error: No valid files found among the arguments '{}'", file_names.join("', '"));
    }
}
