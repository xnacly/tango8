use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::Write,
};

use shared::asm::Instruction;

use crate::config::Config;

#[derive(Debug)]
pub struct Cpu<'cpu> {
    ins: &'cpu [Instruction],
    ac: u8,
    dest: u8,
    pc: u8,
    mem: [u8; 16],
    pub halted: bool,
    config: &'cpu Config,
    dev: HashMap<u8, File>,
}

impl<'cpu> Cpu<'cpu> {
    pub fn new(config: &'cpu Config, ins: &'cpu [Instruction]) -> Self {
        Self {
            ins,
            ac: 0,
            dest: 0,
            pc: 0,
            mem: [0; 16],
            halted: false,
            config,
            dev: config
                .io
                .values()
                .map(|dev| {
                    (
                        dev.addr,
                        OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&dev.file)
                            .expect("Failed to open mapped dev file"),
                    )
                })
                .collect(),
        }
    }

    /// dump val into mem at addr
    fn dump(&mut self, addr: u8, val: u8) -> Option<()> {
        *self.mem.get_mut(addr as usize)? = val;
        if let Some(mut dev) = self.dev.get(&addr) {
            dev.write_all(&[val])
                .expect("Failed to write byte into memory mapped device");
        }
        Some(())
    }

    pub fn step(&mut self) -> Option<()> {
        if self.pc as usize >= self.ins.len() {
            self.halted = true;
            return Some(());
        }

        let cur = &self.ins[self.pc as usize];

        if self.config.verbose {
            println!(
                "{:04x}: 0x{:X} {:>8} (op=0x{:X}, imm=0x{:X}) [ac=0x{:X},dest=0x{:X}]",
                self.pc,
                &cur.encode().unwrap_or(0),
                &cur.to_str_lossy(),
                &cur.op(),
                &cur.imm(),
                self.ac,
                self.dest,
            );
        }

        match cur {
            Instruction::NOP => {}
            Instruction::HALT => self.halted = true,
            Instruction::LOADI { imm } => self.ac = *imm,
            Instruction::MOV => self.dest = self.ac,
            Instruction::ADD => self.ac = self.dest.wrapping_add(self.ac),
            Instruction::SUB => self.ac = self.dest.wrapping_sub(self.ac),
            Instruction::ST { addr } => self.dump(*addr, self.ac)?,
            Instruction::LD { addr } => self.ac = *self.mem.get(*addr as usize)?,
            Instruction::ROL { imm } => self.ac = self.ac.rotate_left((*imm & 0xF) as u32),
        }
        self.pc += 1;

        Some(())
    }
}
