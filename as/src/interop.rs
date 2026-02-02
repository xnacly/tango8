use crate::parser::{Builtin, Node};
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

    fn walk_asm_node(&mut self, node: Node<'ctx>) -> Result<u8, T8Err> {
        match node {
            Node::Literal(node) | Node::Addr(node) => self.walk_asm_node(*node),
            Node::Number(n) => Ok(n),
            Node::Ident { pos, inner } => {
                let u8_as_usize = *self.constants.get(inner).ok_or_else(|| T8Err {
                    line: pos.0,
                    col: pos.1,
                    msg: format!("Undefined identifier {:?}", inner),
                })?;
                Ok(u8::try_from(u8_as_usize).map_err(|_| T8Err {
                    line: pos.0,
                    col: pos.1,
                    msg: format!(
                        "Out of bounds identifier value: `{}`, max size: {}",
                        u8_as_usize,
                        u8::MAX
                    ),
                })?)
            }
            _ => unreachable!(),
        }
    }

    /// used in the assembler for lowering assembly ast to t8 machine code
    pub fn node_to_instruction(&mut self, node: Node<'ctx>) -> Result<Option<Instruction>, T8Err> {
        let i = match node {
            Node::Builtin { kind, lhs, rhs } => {
                match kind {
                    Builtin::Const => {
                        let Node::Number(n) = *rhs else {
                            unreachable!();
                        };
                        self.constants.insert(lhs, n as usize);
                    }
                }
                Ok(None)
            }
            Node::Label { name } => {
                self.constants.insert(name, self.pos);
                Ok(None)
            }
            Node::Instruction { partial, rhs } => {
                let i = if let Some(rhs) = rhs {
                    match partial {
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
                    None => Some(partial.clone()),
                    _ => i,
                })
            }
            _ => unreachable!("{:?}", node),
        };

        self.pos += 1;

        i
    }
}
