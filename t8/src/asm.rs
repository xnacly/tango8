#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    NOP,
    LOADI { imm: u8 },
    MOV,
    ADD,
    SUB,
    ST { addr: u8 },
    ROL { imm: u8 },
    HALT,
}

impl Instruction {
    pub fn encode(&self) -> u8 {
        match self {
            Instruction::NOP => 0x00,
            Instruction::LOADI { imm } => (0x1 << 4) | (imm & 0xF),
            Instruction::MOV => 0x2 << 4,
            Instruction::ADD => 0x3 << 4,
            Instruction::SUB => 0x4 << 4,
            Instruction::ST { addr } => (0x5 << 4) | (addr & 0xF),
            Instruction::ROL { imm } => (0x6 << 4) | (imm & 0xF),
            Instruction::HALT => 0x70,
        }
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
            "ROL" => Self::ROL { imm: 0 },
            "HALT" => Self::HALT,
            _ => return Err(format!("Invalid instruction {:?}", s)),
        })
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
            0x6 => Self::ROL { imm },
            0x7 => Self::HALT,
            _ => return Err("unknown operator"),
        })
    }
}

impl TryFrom<u8> for Instruction {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::decode(value)
    }
}

impl From<Instruction> for u8 {
    fn from(value: Instruction) -> Self {
        value.encode()
    }
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
            Instruction::ROL { imm: 0xF },
            Instruction::HALT,
        ];

        for inst in instructions {
            let encoded: u8 = inst.encode();
            let decoded =
                Instruction::decode(encoded).expect(&format!("Failed to decode {:?}", inst));
            assert_eq!(inst, decoded);
        }
    }
}
