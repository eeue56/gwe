#![allow(irrefutable_let_patterns)]

mod blocks;
mod expressions;
mod generators;
mod parser;
mod tokenizer;

mod cli {
    use super::*;
    use clap::Parser;
    use parser::parser::parse;
    use std::{fs, path::Path, process::Command};

    /// Simple program to greet a person
    #[derive(Parser, Debug, Clone)]
    #[command(author, version, about, long_about = None)]
    pub struct Args {
        /// Name of the person to greet
        #[arg(long)]
        pub file: String,

        #[arg(long, default_value_t = String::from("wat"))]
        pub target: String,

        #[arg(long, default_value_t = false)]
        pub format: bool,

        #[arg(long, default_value_t = false)]
        pub stdout: bool,
    }

    pub fn compile_to_wasm(args: &Args) {
        let original_file_path = &args.file;

        let mut path = Path::new("gwe_build").join(Path::new(&original_file_path));
        path.set_extension("wat");
        let path_as_string = path.as_os_str().to_string_lossy().to_string();

        let mut output_path = path.clone();
        output_path.set_extension("wasm");
        let output_path_as_string = output_path.as_os_str().to_string_lossy().to_string();

        match Command::new("wat2wasm")
            .args([
                &path_as_string.as_str(),
                "-o",
                &output_path_as_string.as_str(),
            ])
            .output()
        {
            Err(err) => println!("Failed to generate wasm: {}", err),
            Ok(value) => {
                if !value.stderr.is_empty() {
                    match std::str::from_utf8(&value.stderr) {
                        Ok(v) => println!("Failed to generate wasm:\n{}", String::from(v)),
                        Err(e) => println!("Invalid UTF-8 sequence in wat2wasm output: {}", e),
                    };
                } else {
                    println!("File written to {}", output_path_as_string);
                }
            }
        }
    }

    pub fn write_file(args: &Args) {
        let output = compile_file(args);

        match output {
            Ok(code) => {
                let original_file_path = &args.file;
                let mut path = Path::new("gwe_build").join(Path::new(&original_file_path));
                path.set_extension("wat");

                let _ = fs::create_dir_all(path.as_path().parent().unwrap());

                match fs::write(path.clone(), code) {
                    Ok(_) => println!(
                        "File written to {}",
                        path.as_os_str().to_string_lossy().to_string()
                    ),
                    Err(error) => println!("Error writing file due to {}", error),
                }
            }
            Err(error) => println!("Not writing file due to {}", error),
        }
    }

    pub fn compile_file(args: &Args) -> Result<String, String> {
        let contents = fs::read_to_string(&args.file);

        match contents {
            Ok(body) => match parse(body) {
                Ok(program) => {
                    println!("Parsed successfully");
                    if args.format {
                        let output = generators::gwe::gwe::generate(program);
                        println!("{}", output);
                        return Ok(output);
                    }
                    match args.target.as_str() {
                        "wat" => {
                            let output = generators::web_assembly::web_assembly::generate(program);
                            Ok(output)
                        }
                        "wasm" => {
                            let output = generators::web_assembly::web_assembly::generate(program);
                            compile_to_wasm(&args);
                            Ok(output)
                        }
                        "gwe" => {
                            let output = generators::gwe::gwe::generate(program);
                            Ok(output)
                        }
                        _ => {
                            let error = format!("Unknown target {}", args.target);
                            println!("{}", error);
                            Err(error)
                        }
                    }
                }
                Err(err) => {
                    let error = format!("Error parsing: {}", err);
                    println!("{}", error);
                    Err(error)
                }
            },
            Err(file_read_error) => {
                let error = format!("Unable to read file due to {}", file_read_error);
                println!("{}", error);
                Err(error)
            }
        }
    }

    pub fn run() {
        let args = Args::parse();

        println!("Compiling file {}", args.file);

        if args.stdout {
            match compile_file(&args) {
                Ok(code) => println!("{}", code),
                Err(_) => (),
            };
        } else {
            write_file(&args);
        }
    }
}

fn main() {
    cli::run();
}

#[cfg(test)]
mod tests {
    use std::fs::{self};

    use super::cli::*;

    #[test]
    fn examples_compile() {
        let files = fs::read_dir("examples/");

        assert!(files.is_ok());

        for file in files.unwrap() {
            match file {
                Ok(entry) => {
                    if entry.path().to_string_lossy().to_string().ends_with("gwe") {
                        match compile_file(&Args {
                            file: entry.path().to_string_lossy().to_string(),
                            target: String::from("gwe"),
                            format: false,
                            stdout: true,
                        }) {
                            Ok(_) => (),
                            Err(err) => panic!("Failed to compile file {:?} due to {}", entry, err),
                        };
                    }
                }
                Err(error) => panic!("Failed to compile file {}", error),
            }
        }
    }
}
