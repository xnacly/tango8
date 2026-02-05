# tango8

Stack of tools around an 8bit cpu inspried by the Intel 8008 and VEB U808

## Overview

- Fixed-length 8-bit instructions
- Memory-mapped I/O
- No interrupts, or stack
- No subroutine calls, no state restoration

See [doc](./doc/isa.md) for ISA and general documentation.

## Tools

- [`as`](./as):  assemble .t8 files into .t8b binary files
- [`dis`](./dis): disassemble .t8b files, roundtrip with `asm`
- [`emu`](./emu): emulate .t8b files

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
```

3. Assemble via `cargo run -p as examples/led.t8`.
4. Execute via `cargo run -p emu examples/led.t8.t8b`.

```text
```

5. Inspect created `led.log` and all bytes send there:

```shell
```

6. Disassemble `led.t8.t8b` via `cargo run -p dis examples/led.t8.t8b`:

```asm
```
