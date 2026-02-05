# t8 ISA

## Registers

| Register | Bits | Description          |
| -------- | ---- | -------------------- |
| A        | 8    | Accumulator          |
| B        | 8    | Registers            |
| IR       | 8    | Instruction register |
| PC       | 8    | Program counter      |

## Instructions

| Instruction    | Bit 7–5 (opcode) | Dst (4) | Src (3) | Bits 2–0  | Notes                     |
| -------------- | ---------------- | ------- | ------- | --------- | ------------------------- |
| `MOV A,B`      | 000              | 0       | 1       | 000       | A=B                       |
| `MOV B,A`      | 000              | 1       | 0       | 000       | B=A                       |
| `ADD A,B`      | 001              | 0       | 1       | 000       | A+=B                      |
| `ADD B,A`      | 001              | 1       | 0       | 000       | B+=A                      |
| `SUB A,B`      | 010              | 0       | 1       | 000       | A-=B                      |
| `SUB B,A`      | 010              | 1       | 0       | 000       | B-=A                      |
| `LOADI A,#imm` | 011              | 0       | 0       | imm (0–7) | Load 3-bit immediate      |
| `LOADI B,#imm` | 011              | 1       | 0       | imm (0–7) | Load 3-bit immediate      |
| `LD A,[B]`     | 100              | 0       | 1       | 000       | Load `A` from `memory[B]` |
| `ST B,[A]`     | 100              | 1       | 0       | 000       | Store `B` to `memory[A]`  |
| `JMP A`        | 101              | 0       | 0       | 000       | Jump to address in `A`    |
| `ROL1 A`       | 110              | 0       | 0       | 000       | Rotate `A` left 1 bit     |
| `HALT`         | 111              | 0       | 0       | 000       | Stop CPU                  |
