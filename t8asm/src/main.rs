use std::fs;

use t8::asm;

mod lexer;
mod parser;

fn main() {
    let input = std::env::args()
        .skip(1)
        .nth(0)
        .expect("No file input given");

    let bytes = fs::read(&input).expect(&format!("Failed to read `{}`", &input));
    let tokens = lexer::Lexer::new(&bytes)
        .lex()
        .expect(&format!("Failed to tokenize contents of `{}`", &input));

    let _ast = parser::Parser::new(&tokens)
        .parse()
        .expect(&format!("Failed to parse contents of `{}`", &input));
}
