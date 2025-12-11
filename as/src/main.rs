#![allow(dead_code)]
use std::{
    fs,
    io::{BufRead, stdout},
    path::Path,
};

use shared::scriptorium::Script;

use crate::interop::Ctx;

mod interop;
mod lexer;
mod parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = std::env::args()
        .nth(1)
        .ok_or_else(|| "Missing .t8 asm file".to_string())?;

    let bytes = fs::read(&input)?;
    let lines = bytes.lines().flatten().collect::<Vec<_>>();
    let tokens = match lexer::Lexer::new(&bytes).lex() {
        Ok(t) => t,
        Err(e) => {
            e.render(&mut stdout(), &lines)?;
            panic!("Failed to tokenize");
        }
    };
    let ast = match parser::Parser::new(&tokens).parse() {
        Ok(a) => a,
        Err(e) => {
            e.render(&mut stdout(), &lines)?;
            panic!("Failed to parse");
        }
    };
    let mut buf = Vec::with_capacity(256);
    let mut ctx = Ctx::new();
    Script::new(&mut buf)?.add_instructions(
        &ast.into_iter()
            .flat_map(|n| {
                ctx.node_to_instruction(n)
                    .map_err(|e| {
                        e.render(&mut stdout(), &lines).unwrap();
                    })
                    .expect("Failed to lower ast to instructions")
            })
            .collect::<Vec<_>>(),
    )?;

    let mut path = Path::new(&input).to_path_buf();
    path.set_extension("t8b");
    fs::write(path, &buf)?;

    Ok(())
}
