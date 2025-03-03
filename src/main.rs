use clap::Parser;
use std::{
    env,
    fs::File,
    io::{self, prelude::*},
    path::{Path, PathBuf},
};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    file_paths: Vec<String>,

    #[arg(short, long, default_value = "false")]
    absolute: bool,

    #[arg(short, long, default_value = "false")]
    recursive: bool,

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

fn read_file(path: &Path) -> io::Result<Option<String>> {
    let mut f = File::open(path)?;
    let mut bytes = Vec::new();
    f.read_to_end(&mut bytes)?;
    match std::str::from_utf8(&bytes) {
        Ok(s) => Ok(Some(s.to_string())),
        Err(_) => Ok(None),
    }
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

fn collect_files(path: &Path, exclude_patterns: &[PathBuf]) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let canonical = resolve_path(path);
    let canonical_str = canonical.to_string_lossy();

    if exclude_patterns
        .iter()
        .any(|pat| canonical_str.contains(&*pat.to_string_lossy()))
    {
        return Ok(files);
    }

    if canonical.is_file() {
        files.push(canonical);
    } else if canonical.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            files.extend(collect_files(&entry_path, exclude_patterns)?);
        }
    }
    Ok(files)
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let is_absolute = args.absolute;
    let exclude_patterns: Vec<PathBuf> = args
        .exclude
        .iter()
        .map(|p| resolve_exclude_pattern(p))
        .collect();
    let mut found_any_file = false;
    let file_names = args.file_paths;

    let mut printed_files = Vec::new();
    let mut excluded_files = Vec::new();
    for file_name in &file_names {
        let path = Path::new(file_name);
        if path.is_dir() {
            if !args.recursive {
                eprintln!(
                    "Warning: {} is a directory (use --recursive to process directories)",
                    file_name
                );
                continue;
            }
            let files_to_process = collect_files(path, &exclude_patterns)?;
            for canonical in files_to_process {
                let display_path = if is_absolute {
                    canonical.clone()
                } else {
                    get_relative_path(&canonical)
                };
                match read_file(&canonical)? {
                    Some(code) => {
                        found_any_file = true;
                        printed_files.push(display_path.clone());
                        print_code(&display_path, &code);
                    }
                    None => {
                        eprintln!(
                            "Warning: {} is not a text file, skipping.",
                            display_path.display()
                        );
                        continue;
                    }
                }
            }
            continue;
        }

        let canonical = resolve_path(path);
        let canonical_str = canonical.to_string_lossy();
        if !exclude_patterns.is_empty()
            && exclude_patterns
                .iter()
                .any(|pat| canonical_str.contains(&*pat.to_string_lossy()))
        {
            let display_path = if is_absolute {
                canonical.clone()
            } else {
                get_relative_path(&canonical)
            };
            excluded_files.push(display_path);
            continue;
        }

        if !path.exists() {
            eprintln!("Warning: {} does not exist", file_name);
            continue;
        }

        if path.is_file() {
            let display_path = if is_absolute {
                canonical.clone()
            } else {
                get_relative_path(&canonical)
            };
            match read_file(&canonical)? {
                Some(code) => {
                    found_any_file = true;
                    printed_files.push(display_path.clone());
                    print_code(&display_path, &code);
                }
                None => {
                    eprintln!(
                        "Warning: {} is not a text file, skipping.",
                        display_path.display()
                    );
                    continue;
                }
            }
            continue;
        }
        if path.is_symlink() {
            eprintln!("Warning: {} is a symlink", file_name);
            continue;
        }
        eprintln!("Warning: {} is not a file", file_name);
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
