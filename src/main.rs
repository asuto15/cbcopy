use clap::Parser;
use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    file_paths: Vec<PathBuf>,

    #[arg(short, long, default_value_t = false)]
    absolute: bool,

    #[arg(short, long, default_value_t = false)]
    recursive: bool,

    #[arg(short, long)]
    exclude: Vec<PathBuf>,
}

fn resolve_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn resolve_exclude_pattern(pattern: &Path) -> PathBuf {
    pattern
        .canonicalize()
        .unwrap_or_else(|_| env::current_dir().unwrap().join(pattern))
}

fn is_excluded(path: &Path, exclude_patterns: &[PathBuf]) -> bool {
    let path_str = path.to_string_lossy();
    exclude_patterns
        .iter()
        .any(|pat| path_str.contains(&*pat.to_string_lossy()))
}

fn read_file(path: &Path) -> io::Result<Option<String>> {
    let bytes = fs::read(path)?;
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
    println!("```{}", path.display());
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
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            files.extend(collect_files(&entry_path, exclude_patterns)?);
        }
    }
    Ok(files)
}

fn process_file(path: &Path, is_absolute: bool) -> io::Result<Option<PathBuf>> {
    let canonical = resolve_path(path);
    if !canonical.exists() {
        eprintln!("Warning: {} does not exist", path.display());
        return Ok(None);
    }
    let display_path = if is_absolute {
        canonical.clone()
    } else {
        get_relative_path(&canonical)
    };

    match read_file(&canonical)? {
        Some(code) => {
            print_code(&display_path, &code);
            Ok(Some(display_path))
        }
        None => {
            eprintln!(
                "Warning: {} is not a text file, skipping.",
                display_path.display()
            );
            Ok(None)
        }
    }
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
    let mut printed_files = Vec::new();
    let mut excluded_files = Vec::new();

    for path in &args.file_paths {
        if path.is_dir() {
            if !args.recursive {
                eprintln!(
                    "Warning: {} is a directory (use --recursive to process directories)",
                    path.display()
                );
                continue;
            }
            let files_to_process = collect_files(path, &exclude_patterns)?;
            for file in files_to_process {
                if let Some(display_path) = process_file(&file, is_absolute)? {
                    printed_files.push(display_path);
                    found_any_file = true;
                }
            }
        } else if path.is_file() {
            let canonical = resolve_path(path);
            if !exclude_patterns.is_empty() && is_excluded(&canonical, &exclude_patterns) {
                let display_path = if is_absolute {
                    canonical.clone()
                } else {
                    get_relative_path(&canonical)
                };
                excluded_files.push(display_path);
                continue;
            }
            if let Some(display_path) = process_file(path, is_absolute)? {
                printed_files.push(display_path);
                found_any_file = true;
            }
        } else if path.is_symlink() {
            eprintln!("Warning: {} is a symlink", path.display());
        } else {
            eprintln!("Warning: {} is not a file", path.display());
        }
    }

    if !excluded_files.is_empty() {
        eprintln!("Excluded files:");
        for file in excluded_files {
            eprintln!("{}", file.display());
        }
    }

    if printed_files.is_empty() {
        eprintln!("Printed files: None");
    } else {
        eprintln!("Printed files:");
        for file in printed_files {
            eprintln!("{}", file.display());
        }
    }

    if !found_any_file {
        let file_list = args
            .file_paths
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join("', '");
        eprintln!(
            "Error: No valid files found among the arguments '{}'",
            file_list
        );
    }

    Ok(())
}
