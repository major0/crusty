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

fn main() {
    println!("Crusty Compiler - Phase 1");
    println!("Infrastructure setup complete!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
