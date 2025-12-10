use std::{fmt::Display, fs};

use t8::{asm::Instruction, mc};

pub struct T8 {
    ins: Vec<Instruction>,
}

impl Display for T8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "; magic={}", String::from_utf8_lossy(mc::MAGIC))?;
        writeln!(f, "; size={}\n", self.ins.len())?;
        for (i, ins) in self.ins.iter().enumerate() {
            let encoded = ins.encode();
            writeln!(
                f,
                "; {:04x}: 0x{:X} (op=0x{:X}, imm=0x{:X})",
                i,
                encoded,
                encoded >> 4,
                encoded & 0xF
            )?;

            match ins {
                Instruction::NOP => writeln!(f, "NOP")?,
                Instruction::LOADI { imm } => writeln!(f, "LOADI {imm}")?,
                Instruction::MOV => writeln!(f, "MOV")?,
                Instruction::ADD => writeln!(f, "ADD")?,
                Instruction::SUB => writeln!(f, "SUB")?,
                Instruction::ST { addr } => writeln!(f, "ST {addr}")?,
                Instruction::ROL { imm } => writeln!(f, "ROL {imm}")?,
                Instruction::HALT => writeln!(f, "HALT")?,
            };
        }

        Ok(())
    }
}

fn from_t8b(bytes: &[u8]) -> Result<T8, Box<dyn std::error::Error>> {
    if &bytes[..5] != mc::MAGIC {
        return Err("Invalid header".into());
    }

    Ok(T8 {
        ins: bytes[5..]
            .iter()
            .map(|b| Instruction::decode(*b).expect("Failed to decode instruction"))
            .collect(),
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = std::env::args()
        .skip(1)
        .next()
        .ok_or_else(|| "Missing .t8 asm file".to_string())?;
    let bytes = fs::read(&input)?;
    let t8 = from_t8b(&bytes)?;
    print!("{t8}");
    Ok(())
}
