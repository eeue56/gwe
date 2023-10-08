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
    use std::fs;

    /// Simple program to greet a person
    #[derive(Parser, Debug)]
    #[command(author, version, about, long_about = None)]
    pub struct Args {
        /// Name of the person to greet
        #[arg(long)]
        pub file: String,

        #[arg(long, default_value_t = String::from("wasm"))]
        pub target: String,

        #[arg(long, default_value_t = false)]
        pub format: bool,
    }

    pub fn compile_file(args: Args) -> Result<String, String> {
        let contents = fs::read_to_string(args.file);

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
                        "wasm" => {
                            let output = generators::web_assembly::web_assembly::generate(program);
                            println!("{}", output);
                            Ok(output)
                        }
                        "gwe" => {
                            let output = generators::gwe::gwe::generate(program);
                            println!("{}", output);
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

        let _ = compile_file(args);
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
                Ok(entry) => match compile_file(Args {
                    file: entry.path().to_string_lossy().to_string(),
                    target: String::from("gwe"),
                    format: false,
                }) {
                    Ok(_) => (),
                    Err(err) => panic!("Failed to compile file {:?} due to {}", entry, err),
                },
                Err(error) => panic!("Failed to compile file {}", error),
            }
        }
    }
}
