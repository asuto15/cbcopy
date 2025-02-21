# cbcopy

`cbcopy` is a command-line tool for developers, enabling quick and efficient sharing of file contents in a code block format.
This command outputs the contents of specified files in Markdown code block format. The file paths can be specified as relative or absolute, and the tool offers an option to choose between displaying relative or absolute paths in the output.

---

## Features

- Outputs the contents of specified files in Markdown code block format.
- Allows users to choose between relative or absolute paths for the output.
- Provides warnings for non-existent files, directories, or symbolic links.
- Excludes the paths from the Output files with command line option.

---

## Usage

### Basic Command

`cbcopy [OPTIONS] <FILE_PATHS>...`

### Options

- `-a`, `--absolute`: Display absolute paths in the output (default is relative paths).
- `-e`, `--exclude` `<EXCLUDE>`: Exclude the paths from the outputs.

### Arguments

- `<FILE_PATHS>`: Specify the paths of files to read (multiple paths are supported).

---

## Examples

### Output contents with relative paths

Run the following command to output the contents of `src/main.rs` with relative paths:

`cbcopy src/main.rs`

Output:
``````
```
// src/main.rs

fn main() {
    println!("Hello, world!");
}
```
``````

### Output contents with absolute paths

Use the `--absolute` option to output the contents with absolute paths:

`cbcopy --absolute src/main.rs`

Output:
``````
```
// /home/user/project/src/main.rs

fn main() {
    println!("Hello, world!");
}
```
``````


### Process multiple files

You can specify multiple files to process them sequentially:

`cbcopy src/main.rs src/lib.rs`


### Exclude paths from outputs

You can select excluded paths not to copy specific files:

`cbcopy src/main.rs -e src/lib.rs`

---

## Error Handling

- If a non-existent file is specified:
  `Warning: missing_file.rs does not exist`

- If a directory is specified:
  `Warning: src/ is a directory`

- If a symbolic link is specified:
  `Warning: link.rs is a symlink`

---

## Installation

1. Clone this repository:

`git clone https://github.com/asuto15/cbcopy.git`
`cd cbcopy`

2. Install using Cargo:

`cargo install --path .`

---

## Development

### Prerequisites

- Rust 1.65.0 or higher
- Cargo

### Build

`cargo build --release`

---

## License

This project is licensed under the [MIT License](LICENSE). See the `LICENSE` file for details.
