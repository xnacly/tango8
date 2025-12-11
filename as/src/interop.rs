use crate::parser::{Builtin, Node};
use shared::{asm::Instruction, err::T8Err};
use std::collections::HashMap;

pub struct Ctx<'ctx> {
    constants: HashMap<&'ctx str, u8>,
}

impl<'ctx> Ctx<'ctx> {
    pub fn new() -> Self {
        Ctx {
            constants: HashMap::new(),
        }
    }

    fn walk_asm_node(&mut self, node: Node<'ctx>) -> Result<u8, T8Err> {
        match node {
            Node::Literal(node) | Node::Addr(node) => self.walk_asm_node(*node),
            Node::Number(n) => Ok(n),
            Node::Ident { pos, inner } => Ok(*self.constants.get(inner).ok_or_else(|| T8Err {
                line: pos.0,
                col: pos.1,
                msg: format!("Undefined identifier `{:?}`", inner),
            })?),
            _ => unreachable!(),
        }
    }

    /// used in the assembler for lowering assembly ast to t8 machine code
    pub fn node_to_instruction(&mut self, node: Node<'ctx>) -> Result<Option<Instruction>, T8Err> {
        match node {
            Node::Builtin { kind, lhs, rhs } => {
                match kind {
                    Builtin::Const => {
                        let Node::Number(n) = *rhs else {
                            unreachable!();
                        };
                        self.constants.insert(lhs, n);
                    }
                }
                Ok(None)
            }
            Node::Instruction { partial, rhs } => {
                let i = match partial {
                    Instruction::LOADI { .. } => Some(Instruction::LOADI {
                        imm: self.walk_asm_node(*rhs.unwrap())?,
                    }),
                    Instruction::ST { .. } => Some(Instruction::ST {
                        addr: self.walk_asm_node(*rhs.unwrap())?,
                    }),
                    Instruction::ROL { .. } => Some(Instruction::ROL {
                        imm: self.walk_asm_node(*rhs.unwrap())?,
                    }),
                    _ => None,
                };

                Ok(match i {
                    None => Some(partial.clone()),
                    _ => i,
                })
            }
            _ => unreachable!("{:?}", node),
        }
    }
}
