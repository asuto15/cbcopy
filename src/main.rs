use clap::Parser;
use std::{env, fs::File, io::prelude::*, path::Path};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    file_paths: Vec<String>,

    #[arg(short, long, default_value = "false")]
    absolute: bool,

    #[arg(short, long)]
    exclude: Vec<String>,
}

fn read_file(path: &Path) -> String {
    let mut f = File::open(path).expect("File not found");

    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("Failed to read file");
    contents
}

fn get_relative_path(path: &Path) -> String {
    let absolute_path = path
        .canonicalize()
        .expect("Failed to resolve absolute path");
    let current_dir = env::current_dir().expect("Failed to get current directory");
    absolute_path
        .strip_prefix(&current_dir)
        .unwrap_or(&absolute_path)
        .to_string_lossy()
        .to_string()
}

fn get_absolute_path(path: &Path) -> String {
    path.canonicalize()
        .expect("Failed to resolve absolute path")
        .to_string_lossy()
        .to_string()
}

fn print_code(path: &str, code: String) {
    println!("```");
    println!("// {}", path);
    print!("{}", code);
    println!("```");
    println!();
}

fn main() {
    let args = Args::parse();

    let is_absolute = args.absolute;
    let exclude_patterns = args.exclude;
    let mut found_any_file = false;
    let file_names = args.file_paths;

    let mut printed_files = Vec::new();
    let mut excluded_files = Vec::new();
    for file_name in &file_names {
        let path = Path::new(&file_name);
        if !exclude_patterns.is_empty() && exclude_patterns.iter().any(|pattern| file_name.contains(pattern)) {
            let path_to_display = match is_absolute {
                true => get_absolute_path(&path),
                false => get_relative_path(&path),
            };
            excluded_files.push(path_to_display.clone());
            continue;
        }

        if path.is_file() {
            found_any_file = true;
            let path_to_display = match is_absolute {
                true => get_absolute_path(&path),
                false => get_relative_path(&path),
            };
            printed_files.push(path_to_display.clone());
            let code = read_file(&path);
            print_code(&path_to_display, code);
        } else if path.is_dir() {
            eprintln!("Warning: {} is a directory", file_name);
        } else if path.is_symlink() {
            eprintln!("Warning: {} is a symlink", file_name);
        } else if path.exists() {
            eprintln!("Warning: {} is not a file", file_name);
        } else {
            eprintln!("Warning: {} does not exist", file_name);
        }
    }
    if !excluded_files.is_empty() {
        eprintln!("Excluded files:");
        for excluded_file in excluded_files {
            eprintln!("{}", excluded_file);
        }
    }

    if printed_files.is_empty() {
        eprintln!("Printed files: None");
    } else {
        eprintln!("Printed files:");
        for printed_file in printed_files {
            eprintln!("{}", printed_file);
        }
    }

    if !found_any_file {
        eprintln!(
            "Error: No valid files found among the arguments '{}'",
            file_names.join("', '")
        );
    }
}
