use crate::parser::{Builtin, InnerNode, Node};
use shared::{asm::Instruction, err::T8Err};
use std::collections::HashMap;

#[derive(Default)]
pub struct Ctx<'ctx> {
    /// keep a mapping of .<label> to its offset to the input start and constants to their numeric
    /// values
    constants: HashMap<&'ctx str, usize>,
    pos: usize,
}

impl<'ctx> Ctx<'ctx> {
    pub fn new() -> Self {
        Self::default()
    }

    fn try_into_8bit_range(n: usize, col: usize, line: usize) -> Result<u8, T8Err> {
        u8::try_from(n).map_err(|_| T8Err {
            line,
            col,
            msg: format!(
                "Out of bounds identifier value: `{}`, max size: {}",
                n,
                u8::MAX
            ),
        })
    }

    fn walk_asm_node(&mut self, node: Node<'ctx>) -> Result<u8, T8Err> {
        let Node { inner, col, line } = node;
        match inner {
            InnerNode::Literal(node) | InnerNode::Addr(node) => self.walk_asm_node(*node),
            InnerNode::Number(n) => Self::try_into_8bit_range(n, col, line),
            InnerNode::Ident(inner) => {
                let u8_as_usize = *self.constants.get(inner).ok_or_else(|| T8Err {
                    line,
                    col,
                    msg: format!("Undefined identifier {:?}", inner),
                })?;
                Self::try_into_8bit_range(u8_as_usize, col, line)
            }
            _ => unreachable!(),
        }
    }

    /// used in the assembler for lowering assembly ast to t8 machine code
    pub fn node_to_instruction(&mut self, node: Node<'ctx>) -> Result<Option<Instruction>, T8Err> {
        let Node { inner, .. } = node;
        let i = match inner {
            InnerNode::Builtin { kind, lhs, rhs } => {
                match kind {
                    Builtin::Const => {
                        let Node {
                            inner: InnerNode::Number(n),
                            ..
                        } = *rhs
                        else {
                            unreachable!();
                        };
                        self.constants.insert(lhs, n as usize);
                    }
                }
                Ok(None)
            }
            InnerNode::Label(name) => {
                self.constants.insert(name, self.pos);
                Ok(None)
            }
            InnerNode::Instruction { name, rhs } => {
                let i = if let Some(rhs) = rhs {
                    match name {
                        Instruction::LOADI { .. } => Some(Instruction::LOADI {
                            imm: self.walk_asm_node(*rhs)?,
                        }),
                        Instruction::ST { .. } => Some(Instruction::ST {
                            addr: self.walk_asm_node(*rhs)?,
                        }),
                        Instruction::LD { .. } => Some(Instruction::LD {
                            addr: self.walk_asm_node(*rhs)?,
                        }),
                        _ => None,
                    }
                } else {
                    None
                };

                Ok(match i {
                    None => Some(name.clone()),
                    _ => i,
                })
            }
            _ => unreachable!("{:?}", inner),
        };

        self.pos += 1;

        i
    }
}
