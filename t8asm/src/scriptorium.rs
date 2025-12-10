use std::collections::HashMap;

use t8::{asm::Instruction, mc};

use crate::parser::{Builtin, Node};

pub struct Script<'script, W: std::io::Write> {
    w: &'script mut W,
    constants: HashMap<&'script str, u8>,
}

impl<'script, W: std::io::Write> Script<'script, W> {
    pub fn new(w: &'script mut W) -> Result<Self, String> {
        w.write(mc::MAGIC).map_err(|e| e.to_string())?;
        Ok(Script {
            w,
            constants: HashMap::new(),
        })
    }

    pub fn walk_node(&mut self, node: &Node<'script>) -> u8 {
        match node {
            Node::Literal(node) | Node::Addr(node) => self.walk_node(node),
            Node::Number(n) => *n,
            Node::Ident(i) => *self.constants.get(i).unwrap(),
            _ => unreachable!(),
        }
    }

    pub fn from_node(&mut self, node: &Node<'script>) -> Result<(), Box<dyn std::error::Error>> {
        match node {
            Node::Builtin { kind, lhs, rhs } => match kind {
                Builtin::Const => {
                    let Node::Number(n) = **rhs else {
                        unreachable!();
                    };
                    self.constants.insert(lhs, n);
                }
            },
            Node::Instruction { partial, rhs } => {
                let i = match partial {
                    Instruction::LOADI { .. } => Some(Instruction::LOADI {
                        imm: self.walk_node(rhs.as_ref().unwrap()),
                    }),
                    Instruction::ST { .. } => Some(Instruction::ST {
                        addr: self.walk_node(rhs.as_ref().unwrap()),
                    }),
                    Instruction::ROL { .. } => Some(Instruction::ROL {
                        imm: self.walk_node(rhs.as_ref().unwrap()),
                    }),
                    _ => None,
                };

                if let Some(i) = i {
                    self.w.write(&[i.encode()])?;
                } else {
                    self.w.write(&[partial.encode()])?;
                };
            }
            _ => unreachable!("{:?}", node),
        };

        Ok(())
    }
}
