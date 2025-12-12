use crate::asm::Instruction;

pub struct Script<'script, W: std::io::Write> {
    w: &'script mut W,
}

pub const MAGIC: &[u8] = b"t8cpu";

impl<'script, W: std::io::Write> Script<'script, W> {
    pub fn new(w: &'script mut W) -> Result<Self, Box<dyn std::error::Error>> {
        w.write_all(MAGIC)?;
        Ok(Script { w })
    }

    /// used in the lisp compiler for lowering instructions to t8 machine code
    pub fn add_instructions(
        &mut self,
        instructions: &[Instruction],
    ) -> Result<(), Box<dyn std::error::Error>> {
        for i in instructions {
            self.w.write_all(&[i
                .encode()
                .ok_or_else(|| "Failed to encode instruction, rhs too large")?])?;
        }
        Ok(())
    }
}

pub fn from(bytes: &[u8]) -> Result<Vec<Instruction>, Box<dyn std::error::Error>> {
    if bytes.len() < 5 {
        return Err("Not enough bytes for a valid t8 binary".into());
    }
    if &bytes[..5] != MAGIC {
        return Err("Invalid header".into());
    }

    Ok(bytes[5..]
        .iter()
        .map(|b| Instruction::decode(*b).expect("Failed to decode instruction"))
        .collect())
}
