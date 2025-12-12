use std::fs;
use std::io::Write;

use shared::asm::Instruction;

fn dis(ins: &[Instruction]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut buf = Vec::new();

    writeln!(
        buf,
        "; magic={}",
        String::from_utf8_lossy(shared::scriptorium::MAGIC)
    )?;

    writeln!(buf, "; size={}\n", ins.len())?;
    for (i, ins) in ins.iter().enumerate() {
        if let Some(encoded) = ins.encode() {
            writeln!(
                buf,
                "; {:04x}: 0x{:X} (op=0x{:X}, imm=0x{:X})",
                i,
                encoded,
                ins.op(),
                ins.imm(),
            )?;

            match ins {
                Instruction::NOP => writeln!(buf, "NOP")?,
                Instruction::LOADI { imm } => writeln!(buf, "LOADI {imm}")?,
                Instruction::MOV => writeln!(buf, "MOV")?,
                Instruction::ADD => writeln!(buf, "ADD")?,
                Instruction::SUB => writeln!(buf, "SUB")?,
                Instruction::ST { addr } => writeln!(buf, "ST {addr}")?,
                Instruction::LD { addr } => writeln!(buf, "LD {addr}")?,
                Instruction::ROL { imm } => writeln!(buf, "ROL {imm}")?,
                Instruction::HALT => writeln!(buf, "HALT")?,
            };
        }
    }

    Ok(buf)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = std::env::args()
        .nth(1)
        .ok_or_else(|| "Missing .t8b binary file".to_string())?;
    let mut handle = std::io::stdout().lock();
    handle.write_all(&dis(&shared::scriptorium::from(&fs::read(&input)?)?)?)?;
    Ok(handle.flush()?)
}
