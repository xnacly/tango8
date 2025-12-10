use t8::asm;

fn main() {
    println!(
        "Hello, world! from the t8cpu emulator: {:?}",
        asm::Instruction::decode(0x0)
    );
}
