#![allow(dead_code)]
use std::fs;

mod lexer;
mod parser;

fn main() {
    let input = std::env::args().next().expect("No file input given");

    let bytes = fs::read(&input).unwrap_or_else(|_| panic!("Failed to read `{}`", &input));
    let tokens = lexer::Lexer::new(&bytes)
        .lex()
        .unwrap_or_else(|_| panic!("Failed to tokenize contents of `{}`", &input));

    let _ast = parser::Parser::new(&tokens)
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse `{}`", &input));
}
