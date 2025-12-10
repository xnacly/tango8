# tango8

> 8-bit accumulator-based CPU and tooling designed for educational and hobbyist hardware projects without going insane. 

## Overview

- Minimal instruction set (7 instructions)
- Single accumulator (AC) + 1 destination register (DEST)
- Fixed-length 8-bit instructions
- Memory-mapped I/O
- No branching, interrupts, or stack in v1 (pure math engine)

See [doc](./doc/isa.md) for ISA documentation.

## Components

- [`asm`](./t8asm) to assemble .t8 files into .t8b binary files
- [`emu`](./t8asm) to emulate .t8b
- [`dis`](./t8dis) to disassemble .t8b files, roundtrip with `asm`

## Usage

1. Define memory mapped device, for instance an LED, see [`t8.toml`](./t8.toml):

```toml
[io]
[io.led] # register a memory mapped LED
addr = 0xF # allow the guest to write to 0xF
file = "led.log" # and forward all writes to led.log
```

2. Write asm interacting with said device (see [examples](./examples)):

```asm
; examples/led.t8
.const led 0xF
.const off 0
.const on 1

; simple on/off LED
    LOADI #on
    ST [led]
    LOADI #off
    ST [led]
    HALT
```

3. Assemble via `cargo run -p t8asm examples/led.t8`.
4. Execute via `cargo run -p t8emu examples/led.t8.t8b`.
5. Inspect created `led.log` and all bytes send there:

```
; led.log
0
1
```

6. Disassemble `led.t8.t8b` via `cargo run -p t8dis examples/led.t8.t8b`:

```asm
; magic=t8cpu
; size=5

; 0000: 0x11 (op=0x1, imm=0x1)
LOADI 1
; 0001: 0x5F (op=0x5, imm=0xF)
ST 15
; 0002: 0x10 (op=0x1, imm=0x0)
LOADI 0
; 0003: 0x5F (op=0x5, imm=0xF)
ST 15
; 0004: 0x70 (op=0x7, imm=0x0)
HALT
```
