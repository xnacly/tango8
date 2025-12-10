#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    NOP,
    LOADI { imm: u8 },
    MOV { dest: u8 },
    ADD { dest: u8 },
    SUB { dest: u8 },
    ST { addr: u8 },
    ROL { imm: u8 },
    HALT,
}

impl Instruction {
    pub fn encode(&self) -> u8 {
        match self {
            Instruction::NOP => 0x00,
            Instruction::LOADI { imm } => (0x1 << 4) | (imm & 0xF),
            Instruction::MOV { dest } => (0x2 << 4) | (dest & 0xF),
            Instruction::ADD { dest } => (0x3 << 4) | (dest & 0xF),
            Instruction::SUB { dest } => (0x4 << 4) | (dest & 0xF),
            Instruction::ST { addr } => (0x5 << 4) | (addr & 0xF),
            Instruction::ROL { imm } => (0x6 << 4) | (imm & 0xF),
            Instruction::HALT => 0x70,
        }
    }

    pub fn decode(b: u8) -> Result<Self, &'static str> {
        let op = b >> 4;
        let imm = b & 0xF;
        Ok(match op {
            0x0 => Self::NOP,
            0x1 => Self::LOADI { imm },
            0x2 => Self::MOV { dest: imm },
            0x3 => Self::ADD { dest: imm },
            0x4 => Self::SUB { dest: imm },
            0x5 => Self::ST { addr: imm },
            0x6 => Self::ROL { imm },
            0x7 => Self::HALT,
            _ => return Err("unkown operator"),
        })
    }
}

impl TryFrom<u8> for Instruction {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::decode(value)
    }
}

impl Into<u8> for Instruction {
    fn into(self) -> u8 {
        self.encode()
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
            Instruction::MOV { dest: 4 },
            Instruction::ADD { dest: 2 },
            Instruction::SUB { dest: 7 },
            Instruction::ST { addr: 0xF },
            Instruction::ROL { imm: 0xF },
            Instruction::HALT,
        ];

        for inst in instructions {
            let encoded: u8 = inst.clone().encode();
            let decoded = Instruction::decode(encoded).expect("Failed to decode");
            assert_eq!(inst, decoded);
        }
    }
}
