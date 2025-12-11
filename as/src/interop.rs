use std::collections::HashMap;

use shared::asm::Instruction;

use crate::parser::{Builtin, Node};

pub struct Ctx<'ctx> {
    constants: HashMap<&'ctx str, u8>,
}

impl<'ctx> Ctx<'ctx> {
    pub fn new() -> Self {
        Ctx {
            constants: HashMap::new(),
        }
    }

    fn walk_asm_node(&mut self, node: Node<'ctx>) -> u8 {
        match node {
            Node::Literal(node) | Node::Addr(node) => self.walk_asm_node(*node),
            Node::Number(n) => n,
            Node::Ident(i) => *self
                .constants
                .get(i)
                .unwrap_or_else(|| panic!("Undefined identifier `{:?}`", i)),
            _ => unreachable!(),
        }
    }

    /// used in the assembler for lowering assembly ast to t8 machine code
    pub fn node_to_instruction(&mut self, node: Node<'ctx>) -> Option<Instruction> {
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
                None
            }
            Node::Instruction { partial, rhs } => {
                let i = match partial {
                    Instruction::LOADI { .. } => Some(Instruction::LOADI {
                        imm: self.walk_asm_node(*rhs.unwrap()),
                    }),
                    Instruction::ST { .. } => Some(Instruction::ST {
                        addr: self.walk_asm_node(*rhs.unwrap()),
                    }),
                    Instruction::ROL { .. } => Some(Instruction::ROL {
                        imm: self.walk_asm_node(*rhs.unwrap()),
                    }),
                    _ => None,
                };

                match i {
                    None => Some(partial.clone()),
                    _ => i,
                }
            }
            _ => unreachable!("{:?}", node),
        }
    }
}
