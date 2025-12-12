use std::fs;

mod config;
mod cpu;

use shared::scriptorium::from;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = std::env::args()
        .nth(1)
        .ok_or_else(|| "Missing .t8b binary file".to_string())?;

    let config =
        toml::from_slice(&fs::read("t8.toml")?).expect("Failed to parse t8.toml configuration");

    let instructions = from(&fs::read(&input)?)?;
    let mut cpu = cpu::Cpu::new(&config, &instructions);
    while !cpu.halted {
        if cpu.step().is_none() {
            panic!("Failed to walk cpu")
        }
    }
    Ok(())
}
