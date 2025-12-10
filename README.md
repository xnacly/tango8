# tango8

> 8-bit accumulator-based CPU and tooling designed for educational and hobbyist hardware projects without going insane. 

## Overview

- Minimal instruction set (7 instructions)
- Single accumulator (AC) + 1 destination register (DEST)
- Fixed-length 8-bit instructions
- Memory-mapped I/O
- No branching, interrupts, or stack in v1 (pure math engine)

See [doc](./doc) for ISA documentation.

## Components

- [`asm`](./t8asm) to assemble .t8 files into .t8b binary files
- [`emu`](./t8asm) to emulate .t8b
