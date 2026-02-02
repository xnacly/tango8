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
- 4-bit immediate values for `LOADI`, `ST` and `LD`
- Memory addresses: 8 bits
- Memory-mapped I/O: addresses, e.g. 0xF for LED

## Instructions

| Mnemonic | Opcode | Operand | Description               |
| -------- | ------ | ------- | ------------------------- |
| NOP      | 0x0    | -       | No operation              |
| LOADI    | 0x1    | imm     | Load immediate into AC    |
| MOV      | 0x2    | -       | AC -> DEST                |
| ADD      | 0x3    | -       | DEST += AC                |
| SUB      | 0x4    | -       | DEST -= AC                |
| ST       | 0x5    | addr    | write AC into addr        |
| LD       | 0x6    | imm     | load byte at addr into AC |
| ROL1     | 0x7    | -       | Rotate AC left by 1 bit   |
| HALT     | 0x8    | -       | Stop CPU                  |
| JMP      | 0x9    | -       | PC = AC                  |

**Instruction encoding:**

- Upper 4 bits = opcode
- Lower 4 bits = immediate / addr
- Other instructions ignore the lower 4 bits
