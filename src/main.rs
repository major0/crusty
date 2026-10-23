// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

mod ast;
mod cli;
mod codegen;
mod error;
mod parser;
mod pretty;
mod rustc;
mod semantic;
mod test_cmd;
mod utils;

use cli::{run_compiler, CompilerOptions};
use std::process;

fn main() {
    // `crusty test [args…]` is a subcommand that delegates to `cargo test`
    // (ADR-0006). Intercept it before clap parses the single-shot transpiler
    // CLI, so the existing `crustyc <input>` form is untouched.
    let argv: Vec<String> = std::env::args().collect();
    if argv.get(1).map(String::as_str) == Some("test") {
        let code = test_cmd::run_test(&argv[2..]);
        process::exit(code);
    }

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
