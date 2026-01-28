// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

mod cli;
mod lexer;
mod parser;
mod ast;
mod semantic;
mod codegen;
mod error;
mod utils;
mod pretty;

use cli::{CompilerOptions, run_compiler};
use std::process;

fn main() {
    // Parse command-line arguments
    let options = CompilerOptions::parse_args();

    // Run the compiler
    match run_compiler(&options) {
        Ok(()) => {
            if options.verbose {
                println!("Compilation completed successfully");
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
