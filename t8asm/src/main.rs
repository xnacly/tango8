#![allow(dead_code)]
use std::{fs, io::Write};

use crate::scriptorium::Script;

mod lexer;
mod parser;
/// codegen to lower the asm to t8 machinecode
mod scriptorium;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = std::env::args()
        .skip(1)
        .next()
        .ok_or_else(|| "Missing .t8 asm file".to_string())?;

    let bytes = fs::read(&input)?;
    let tokens = lexer::Lexer::new(&bytes).lex()?;
    let ast = parser::Parser::new(&tokens).parse()?;
    let mut buf = Vec::with_capacity(256);
    let mut script = Script::new(&mut buf)?;
    for node in ast {
        script.from_node(&node)?
    }

    input.push_str(".t8b");
    fs::write(input, &buf)?;

    Ok(())
}
