// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Crusty compiler library

pub mod ast;
pub mod cli;
pub mod codegen;
#[cfg(test)]
mod codegen_properties;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod pretty;
#[cfg(test)]
mod pretty_properties;
pub mod semantic;
pub mod utils;
