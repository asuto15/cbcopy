use clap::Parser;
use std::{
    env, fs, io,
    collections::BTreeMap,
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

fn detect_language(path: &Path) -> Option<&'static str> {
    match path.extension().and_then(|ext| ext.to_str()).unwrap_or("").to_lowercase().as_str() {
        "rs" => Some("rust"),
        "py" => Some("python"),
        "js" => Some("javascript"),
        "ts" => Some("typescript"),
        "c" | "h" => Some("c"),
        "cpp" | "c++" | "cc" | "cp" | "cxx" | "hpp" | "h++" | "hh" | "hxx" => Some("cpp"),
        "cs" => Some("csharp"),
        "go" => Some("go"),
        "rb" => Some("ruby"),
        "java" => Some("java"),
        "kt" => Some("kotlin"),
        "php" => Some("php"),
        "swift" => Some("swift"),
        "dart" => Some("dart"),
        "jl" => Some("julia"),
        "sql" => Some("sql"),
        "sh" => Some("bash"),
        "html" | "htm" => Some("html"),
        "css" => Some("css"),
        "json" => Some("json"),
        "xml" => Some("xml"),
        "yaml" | "yml" => Some("yaml"),
        "md" => Some("markdown"),
        "tex" => Some("latex"),
        "vim" => Some("vim"),
        _ => None,
    }
}

fn print_code(path: &Path, code: &str) {
    let language = detect_language(path).unwrap_or("plaintext");
    println!("```{}:{}", language, path.display());
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

#[derive(Default)]
struct TreeNode {
    children: BTreeMap<String, TreeNode>,
    is_file: bool,
}

fn path_components(path: &Path) -> Vec<String> {
    use std::path::Component;

    let mut components = Vec::new();
    for comp in path.components() {
        let part = match comp {
            Component::RootDir => "/".to_string(),
            Component::Prefix(prefix) => prefix.as_os_str().to_string_lossy().to_string(),
            Component::CurDir => ".".to_string(),
            Component::ParentDir => "..".to_string(),
            Component::Normal(name) => name.to_string_lossy().to_string(),
        };
        components.push(part);
    }
    components
}

fn build_tree(paths: &[PathBuf]) -> TreeNode {
    let mut root = TreeNode::default();
    for path in paths {
        let parts = path_components(path);
        if parts.is_empty() {
            continue;
        }
        let mut node = &mut root;
        for (idx, part) in parts.iter().enumerate() {
            let is_last = idx + 1 == parts.len();
            node = node.children.entry(part.clone()).or_default();
            if is_last {
                node.is_file = true;
            }
        }
    }
    root
}

fn render_tree(node: &TreeNode, prefix: &str, lines: &mut Vec<String>) {
    let total = node.children.len();
    for (idx, (name, child)) in node.children.iter().enumerate() {
        let is_last = idx + 1 == total;
        let connector = if is_last { "└── " } else { "├── " };
        lines.push(format!("{}{}{}", prefix, connector, name));
        let next_prefix = if is_last {
            format!("{}    ", prefix)
        } else {
            format!("{}│   ", prefix)
        };
        if !child.children.is_empty() {
            render_tree(child, &next_prefix, lines);
        }
    }
}

fn tree_lines(paths: &[PathBuf], is_absolute: bool) -> Vec<String> {
    let root = build_tree(paths);
    let mut lines = Vec::new();
    if is_absolute {
        if root.children.len() == 1 {
            if let Some((name, child)) = root.children.iter().next() {
                lines.push(name.clone());
                render_tree(child, "", &mut lines);
                return lines;
            }
        }
        render_tree(&root, "", &mut lines);
    } else {
        lines.push(".".to_string());
        render_tree(&root, "", &mut lines);
    }
    lines
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
        let lines = tree_lines(&printed_files, is_absolute);
        for line in lines {
            eprintln!("{}", line);
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
