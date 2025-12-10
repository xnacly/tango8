# t8 ISA

## Registers

| Register | Bits | Description          |
| -------- | ---- | -------------------- |
| AC       | 8    | Accumulator          |
| DEST     | 8    | Destination register |
| IR       | 8    | Instruction register |
| PC       | 8    | Program counter      |

## Data

- 8-bit registers
- 4-bit immediate values for `LOADI`
- Memory addresses: 8 bits
- Memory-mapped I/O: addresses, e.g. 0xF for LED

## Instructions

| Mnemonic | Opcode | Operand | Description                    |
| -------- | ------ | ------- | ------------------------------ |
| NOP      | 0x0    | -       | No operation                   |
| LOADI    | 0x1    | imm     | Load immediate into AC         |
| MOV      | 0x2    | dest    | AC -> DEST                     |
| ADD      | 0x3    | dest    | DEST += AC                     |
| SUB      | 0x4    | dest    | DEST -= AC                     |
| ST       | 0x5    | addr    | AC -> memory-mapped I/O (LEDs) |
| ROL      | 0x6    | imm     | Rotate AC left by `imm` bits   |
| HALT     | 0x7    | -       | Stop CPU                       |

**Instruction encoding:**

- Upper 4 bits = opcode
- Lower 4 bits = immediate (for `LOADI` and `ROL`)
- Other instructions ignore the lower 4 bits
