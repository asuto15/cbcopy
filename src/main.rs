use clap::Parser;
use std::{env, fs::File, io::{self, prelude::*}, path::{Path, PathBuf}};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    file_paths: Vec<String>,

    #[arg(short, long, default_value = "false")]
    absolute: bool,

    #[arg(short, long)]
    exclude: Vec<String>,
}

fn resolve_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn resolve_exclude_pattern(pattern: &str) -> PathBuf {
    let p = Path::new(pattern);
    p.canonicalize().unwrap_or_else(|_| {
        let current_dir = env::current_dir().unwrap();
        current_dir.join(p)
    })
}

fn read_file(path: &Path) -> io::Result<String> {
    let mut f = File::open(path)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    Ok(contents)
}

fn get_relative_path(canonical: &Path) -> PathBuf {
    let current_dir = env::current_dir().unwrap();
    canonical
        .strip_prefix(&current_dir)
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|_| canonical.to_path_buf())
}

fn print_code(path: &Path, code: &str) {
    println!("```");
    println!("// {}", path.display());
    print!("{}", code);
    println!("```");
    println!();
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let is_absolute = args.absolute;
    let exclude_patterns: Vec<PathBuf> = args.exclude.iter().map(|p| resolve_exclude_pattern(p)).collect();
    let mut found_any_file = false;
    let file_names = args.file_paths;

    let mut printed_files = Vec::new();
    let mut excluded_files = Vec::new();
    for file_name in &file_names {
        let path = Path::new(file_name);
        let canonical = resolve_path(path);
        let display_path = if is_absolute {
            canonical.clone()
        } else {
            get_relative_path(&canonical)
        };

        if !exclude_patterns.is_empty()
            && exclude_patterns
                .iter()
                .any(|pat| canonical.to_string_lossy().contains(&*pat.to_string_lossy()))
        {
            excluded_files.push(display_path);
            continue;
        }

        if path.is_file() {
            found_any_file = true;
            printed_files.push(display_path.clone());
            let code = read_file(&canonical)?;
            print_code(&display_path, &code);
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
            eprintln!("{}", excluded_file.display());
        }
    }

    if printed_files.is_empty() {
        eprintln!("Printed files: None");
    } else {
        eprintln!("Printed files:");
        for printed_file in printed_files {
            eprintln!("{}", printed_file.display());
        }
    }

    if !found_any_file {
        eprintln!(
            "Error: No valid files found among the arguments '{}'",
            file_names.join("', '")
        );
    }

    Ok(())
}
