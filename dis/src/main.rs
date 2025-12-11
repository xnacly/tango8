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
        let encoded = ins.encode();
        writeln!(
            buf,
            "; {:04x}: 0x{:X} (op=0x{:X}, imm=0x{:X})",
            i,
            encoded,
            encoded >> 4,
            encoded & 0xF
        )?;

        match ins {
            Instruction::NOP => writeln!(buf, "NOP")?,
            Instruction::LOADI { imm } => writeln!(buf, "LOADI {imm}")?,
            Instruction::MOV => writeln!(buf, "MOV")?,
            Instruction::ADD => writeln!(buf, "ADD")?,
            Instruction::SUB => writeln!(buf, "SUB")?,
            Instruction::ST { addr } => writeln!(buf, "ST {addr}")?,
            Instruction::ROL { imm } => writeln!(buf, "ROL {imm}")?,
            Instruction::HALT => writeln!(buf, "HALT")?,
        };
    }

    Ok(buf)
}

fn from_binary(bytes: &[u8]) -> Result<Vec<Instruction>, Box<dyn std::error::Error>> {
    if bytes.len() < 5 {
        return Err("Not enough bytes for a valid t8 binary".into());
    }
    if &bytes[..5] != shared::scriptorium::MAGIC {
        return Err("Invalid header".into());
    }

    Ok(bytes[5..]
        .iter()
        .map(|b| Instruction::decode(*b).expect("Failed to decode instruction"))
        .collect())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = std::env::args()
        .nth(1)
        .ok_or_else(|| "Missing .t8b binary file".to_string())?;
    let mut handle = std::io::stdout().lock();
    handle.write_all(&dis(&from_binary(&fs::read(&input)?)?)?)?;
    Ok(handle.flush()?)
}
