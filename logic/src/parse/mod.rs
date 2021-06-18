//! The parser.
//!
//! Parsing is conducted in a single step (rather than first lexing, and then parsing) for
//! efficiency.
//!
//! Thanks to @kevin-brown for writing this formal grammar
//! https://github.com/pallets/jinja/blob/GH-1194-formal-grammar/grammar.ebnf
//!
//! todo: investigate using SIMD for faster parsing
//! todo: better error messages

mod ast;
mod call;
mod r#else;
mod expr;
mod filter;
mod r#for;
mod ident;
mod r#if;
mod literal;
mod r#macro;
mod set;
mod stmt;
mod template;
mod utils;

pub(crate) use utils::*;

pub use template::Template;
pub use utils::{Parse, ParseError, ParseResult};
