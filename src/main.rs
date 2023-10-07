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
    struct Args {
        /// Name of the person to greet
        #[arg(long)]
        file: String,

        #[arg(long, default_value_t = String::from("wasm"))]
        target: String,
    }

    pub fn run() {
        let args = Args::parse();

        println!("Compiling file {}", args.file);

        let contents = fs::read_to_string(args.file);

        match contents {
            Ok(body) => match parse(body) {
                Ok(program) => {
                    println!("Parsed successfully");
                    match args.target.as_str() {
                        "wasm" => {
                            let output = generators::web_assembly::web_assembly::generate(program);
                            println!("{}", output);
                        }
                        "gwe" => {
                            let output = generators::gwe::gwe::generate(program);
                            println!("{}", output);
                        }
                        _ => println!("Unknown target {}", args.target),
                    }
                }
                Err(err) => println!("Error parsing: {}", err),
            },
            Err(file_read_error) => println!("Unable to read file due to {}", file_read_error),
        }
    }
}

fn main() {
    cli::run();
}
