use std::fs;
use std::io::Write;

use shared::asm::Instruction;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = std::env::args()
        .nth(1)
        .ok_or_else(|| "Missing .t8b binary file".to_string())?;
    let mut handle = std::io::stdout().lock();
    handle.write_all(&shared::asm::dis(&shared::scriptorium::from(&fs::read(
        &input,
    )?)?)?)?;
    Ok(handle.flush()?)
}
