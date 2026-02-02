// Copyright (c) 2026 Mark Ferrell
// Licensed under the MIT License. See LICENSE.txt in the project root.

//! Crusty compiler library

pub mod ast;
#[cfg(test)]
mod c_style_declaration_tests;
pub mod cli;
#[cfg(test)]
mod cli_properties;
pub mod codegen;
#[cfg(test)]
mod codegen_advanced_tests;
#[cfg(test)]
mod codegen_bitwise_tests;
#[cfg(test)]
mod codegen_crusty_advanced_tests;
#[cfg(test)]
mod codegen_crusty_tests;
#[cfg(test)]
mod codegen_properties;
pub mod error;
#[cfg(test)]
mod error_coverage_tests;
pub mod lexer;
#[cfg(test)]
mod lexer_coverage_tests;
#[cfg(test)]
mod nested_function_tests;
pub mod parser;
#[cfg(test)]
mod parser_additional_coverage_tests;
#[cfg(test)]
mod parser_advanced_tests;
#[cfg(test)]
mod parser_coverage_tests;
#[cfg(test)]
mod parser_edge_case_tests;
#[cfg(test)]
mod parser_error_tests;
#[cfg(test)]
mod parser_properties;
pub mod pretty;
#[cfg(test)]
mod pretty_properties;
pub mod rustc;
#[cfg(test)]
mod rustc_integration_tests;
pub mod semantic;
#[cfg(test)]
mod semantic_additional_tests;
#[cfg(test)]
mod semantic_advanced_tests;
#[cfg(test)]
mod semantic_coverage_tests;
#[cfg(test)]
mod semantic_expression_tests;
#[cfg(test)]
mod semantic_return_tests;
#[cfg(test)]
mod semantic_statement_tests;
#[cfg(test)]
mod semantic_type_tests;
#[cfg(test)]
mod typedef_integration_tests;
pub mod utils;
