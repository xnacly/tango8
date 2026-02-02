use std::io::Write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    NOP,
    LOADI { imm: u8 },
    MOV,
    ST { addr: u8 },
    LD { addr: u8 },
    ADD,
    SUB,
    ROL1,
    HALT,
    JMP,
}

impl Instruction {
    pub fn encode(&self) -> Option<u8> {
        Some(match self {
            Instruction::NOP => 0x00,
            Instruction::LOADI { imm } => {
                (0x1 << 4) | if *imm > 0xF { return None } else { imm & 0xF }
            }
            Instruction::MOV => 0x2 << 4,
            Instruction::ADD => 0x3 << 4,
            Instruction::SUB => 0x4 << 4,
            Instruction::ST { addr } => {
                (0x5 << 4) | if *addr > 0xF { return None } else { addr & 0xF }
            }
            Instruction::LD { addr } => {
                (0x6 << 4) | if *addr > 0xF { return None } else { addr & 0xF }
            }
            Instruction::ROL1 => 0x70,
            Instruction::HALT => 0x80,
            Instruction::JMP => 0x90,
        })
    }

    /// this is lossy, meaning there is only support for looking instructions up by their textual
    /// representation, for instance LOADI. LOADI #5 will result in "Invalid instruction".
    pub fn from_str_lossy(s: &str) -> Result<Self, String> {
        Ok(match s {
            "NOP" => Self::NOP,
            "LOADI" => Self::LOADI { imm: 0 },
            "MOV" => Self::MOV,
            "ADD" => Self::ADD,
            "SUB" => Self::SUB,
            "ST" => Self::ST { addr: 0 },
            "LD" => Self::LD { addr: 0 },
            "ROL1" => Self::ROL1,
            "HALT" => Self::HALT,
            "JMP" => Self::JMP,
            _ => return Err(format!("Invalid instruction {:?}", s)),
        })
    }

    pub fn to_str_lossy(&self) -> &'static str {
        match self {
            Self::NOP => "NOP",
            Self::LOADI { .. } => "LOADI",
            Self::MOV => "MOV",
            Self::ADD => "ADD",
            Self::SUB => "SUB",
            Self::ST { .. } => "ST",
            Self::LD { .. } => "LD",
            Self::ROL1 => "ROL1",
            Self::HALT => "HALT",
            Self::JMP => "JMP",
        }
    }

    pub fn decode(b: u8) -> Result<Self, &'static str> {
        let op = b >> 4;
        let imm = b & 0xF;
        Ok(match op {
            0x0 => Self::NOP,
            0x1 => Self::LOADI { imm },
            0x2 => Self::MOV,
            0x3 => Self::ADD,
            0x4 => Self::SUB,
            0x5 => Self::ST { addr: imm },
            0x6 => Self::LD { addr: imm },
            0x7 => Self::ROL1,
            0x8 => Self::HALT,
            0x9 => Self::JMP,
            _ => return Err("unknown operator"),
        })
    }

    pub fn op(&self) -> u8 {
        match self {
            Instruction::NOP => 0x00,
            Instruction::LOADI { .. } => 0x1 << 4,
            Instruction::MOV => 0x2 << 4,
            Instruction::ADD => 0x3 << 4,
            Instruction::SUB => 0x4 << 4,
            Instruction::ST { .. } => 0x5 << 4,
            Instruction::LD { .. } => 0x6 << 4,
            Instruction::ROL1 => 0x70,
            Instruction::HALT => 0x80,
            Instruction::JMP => 0x90,
        }
    }

    pub fn imm(&self) -> u8 {
        (match self {
            Self::ST { addr } | Self::LD { addr } => *addr,
            Self::LOADI { imm } => *imm,
            _ => 0,
        }) & 0xF
    }
}

impl TryFrom<u8> for Instruction {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::decode(value)
    }
}

impl TryFrom<Instruction> for u8 {
    type Error = &'static str;

    fn try_from(value: Instruction) -> Result<Self, Self::Error> {
        value
            .encode()
            .ok_or_else(|| "Failed to encode instruction, rhs too large")
    }
}

pub fn dis(ins: &[Instruction]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut buf = Vec::new();

    writeln!(
        buf,
        "; magic={}",
        String::from_utf8_lossy(crate::scriptorium::MAGIC)
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
                Instruction::ROL1 => writeln!(buf, "ROL1")?,
                Instruction::HALT => writeln!(buf, "HALT")?,
                Instruction::JMP => writeln!(buf, "JMP")?,
            };
        }
    }

    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::Instruction;

    #[test]
    fn test_encode_decode_roundtrip() {
        let instructions = [
            Instruction::NOP,
            Instruction::LOADI { imm: b'\n' },
            Instruction::MOV,
            Instruction::ADD,
            Instruction::SUB,
            Instruction::ST { addr: 0xF },
            Instruction::LD { addr: 0x4 },
            Instruction::ROL1,
            Instruction::JMP,
            Instruction::HALT,
        ];

        for inst in instructions {
            let encoded: u8 = inst.encode().unwrap();
            let decoded =
                Instruction::decode(encoded).expect(&format!("Failed to decode {:?}", inst));
            assert_eq!(inst, decoded);
        }
    }
}
