use crate::lexer::Token;
use shared::asm::Instruction;

#[derive(Debug, PartialEq, Eq)]
pub enum Builtin {
    Const,
}

impl TryFrom<&[u8]> for Builtin {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            b"const" => Ok(Self::Const),
            _ => Err("Unknown builtin".into()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Node<'node> {
    /// .<kind> <lhs> <rhs>
    Builtin {
        kind: Builtin,
        lhs: &'node str,
        rhs: Box<Node<'node>>,
    },
    /// <instruction> <rhs>
    Instruction {
        /// partial since this does not include inner values, only the name -> instruction lookup
        /// is done at this point
        partial: Instruction,
        rhs: Option<Box<Node<'node>>>,
    },
    /// #<literal>
    Literal(Box<Node<'node>>),
    /// [<addr>]
    Addr(Box<Node<'node>>),
    Number(u8),
    Ident(&'node str),
}

pub struct Parser<'parser> {
    src: &'parser [Token<'parser>],
    pos: usize,
}

impl<'parser> Parser<'parser> {
    pub fn new(src: &'parser [Token<'parser>]) -> Self {
        Parser { src, pos: 0 }
    }

    fn end(&self) -> bool {
        self.pos >= self.src.len()
    }

    fn cur(&self) -> Option<&Token<'parser>> {
        self.src.get(self.pos)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn parse_one(&mut self) -> Result<Node<'parser>, String> {
        Ok(match self.cur().copied() {
            Some(Token::Builtin(name)) => {
                // skip .<kind>
                self.advance();
                let lhs = if let Some(Token::Ident(lhs)) = self.cur() {
                    str::from_utf8(lhs).unwrap()
                } else {
                    return Err("Wanted ident as builtin lhs, got something else".into());
                };
                // skip lhs
                self.advance();

                let kind = (*name).try_into()?;
                let rhs = match kind {
                    Builtin::Const => match self.cur() {
                        Some(Token::Number(n)) => Node::Number(*n),
                        _ => {
                            return Err(format!(
                                "Invalid rhs for .const {:?}, wanted number",
                                self.cur()
                            ));
                        }
                    },
                };

                // skip argument
                self.advance();

                Node::Builtin {
                    kind,
                    lhs,
                    rhs: Box::new(rhs),
                }
            }
            Some(Token::Ident(ident)) => {
                // skip self
                self.advance();
                let partial = Instruction::from_str_lossy(str::from_utf8(ident).unwrap())?;
                let rhs = match &partial {
                    Instruction::LOADI { .. }
                    | Instruction::ST { .. }
                    | Instruction::ROL { .. } => Some(Box::new(self.parse_one()?)),
                    _ => None,
                };
                Node::Instruction { partial, rhs }
            }
            Some(Token::Hash) => {
                // skip #
                self.advance();
                let inner = match self.cur() {
                    Some(Token::Number(n)) => Node::Number(*n),
                    Some(Token::Ident(ident)) => Node::Ident(str::from_utf8(ident).unwrap()),
                    _ => {
                        return Err(format!(
                            "Invalid inner literal {:?}, wanted ident or number",
                            self.cur()
                        ));
                    }
                };
                // skip number or ident
                self.advance();
                Node::Literal(Box::new(inner))
            }
            Some(Token::LeftBraket) => {
                // skip [
                self.advance();
                let inner = match self.cur() {
                    Some(Token::Number(n)) => Node::Number(*n),
                    Some(Token::Ident(ident)) => Node::Ident(str::from_utf8(ident).unwrap()),
                    _ => {
                        return Err(format!(
                            "Invalid inner addr {:?}, wanted ident or number",
                            self.cur()
                        ));
                    }
                };
                // skip inner
                self.advance();

                let addr = Node::Addr(Box::new(inner));
                if self.cur() != Some(&Token::RightBraket) {
                    return Err("] needed for addr syntax".into());
                }

                // skip ]
                self.advance();
                addr
            }
            Some(Token::Number(n)) => {
                self.advance();
                Node::Number(n)
            }
            _ => return Err(format!("Unkown token type {:?}", self.cur())),
        })
    }

    pub fn parse(&mut self) -> Result<Vec<Node<'parser>>, String> {
        let mut r = vec![];
        while !self.end() {
            r.push(self.parse_one()?);
        }
        Ok(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use shared::asm::Instruction;

    #[test]
    fn test_const_builtin() {
        let tokens = Lexer::new(".const led 5".as_bytes())
            .lex()
            .expect("lex failed");

        let ast = Parser::new(&tokens).parse().expect("parse failed");

        assert_eq!(
            ast,
            vec![Node::Builtin {
                kind: Builtin::Const,
                lhs: "led",
                rhs: Box::new(Node::Number(5)),
            }]
        );
    }

    #[test]
    fn test_loadi_literal_ident() {
        let tokens = Lexer::new("LOADI #foo".as_bytes())
            .lex()
            .expect("lex failed");
        let ast = Parser::new(&tokens).parse().expect("parse failed");

        assert_eq!(
            ast,
            vec![Node::Instruction {
                partial: Instruction::LOADI { imm: 0 },
                rhs: Some(Box::new(Node::Literal(Box::new(Node::Ident("foo"))))),
            }]
        );
    }

    #[test]
    fn test_loadi_literal_number() {
        let tokens = Lexer::new("LOADI #3".as_bytes()).lex().expect("lex failed");
        let ast = Parser::new(&tokens).parse().expect("parse failed");

        assert_eq!(
            ast,
            vec![Node::Instruction {
                partial: Instruction::LOADI { imm: 0 },
                rhs: Some(Box::new(Node::Literal(Box::new(Node::Number(3))))),
            }]
        );
    }

    #[test]
    fn test_st_addr_ident() {
        let tokens = Lexer::new("ST [led]".as_bytes()).lex().expect("lex failed");
        let ast = Parser::new(&tokens).parse().expect("parse failed");

        assert_eq!(
            ast,
            vec![Node::Instruction {
                partial: Instruction::ST { addr: 0 },
                rhs: Some(Box::new(Node::Addr(Box::new(Node::Ident("led"))))),
            }]
        );
    }

    #[test]
    fn test_st_addr_number() {
        let tokens = Lexer::new("ST [5]".as_bytes()).lex().expect("lex failed");
        let ast = Parser::new(&tokens).parse().expect("parse failed");

        assert_eq!(
            ast,
            vec![Node::Instruction {
                partial: Instruction::ST { addr: 0 },
                rhs: Some(Box::new(Node::Addr(Box::new(Node::Number(5))))),
            }]
        );
    }

    #[test]
    fn test_small_program() {
        let src = "
.const led 0xF
LOADI #led
ST [led]
HALT
        ";

        let tokens = Lexer::new(src.as_bytes()).lex().expect("lex failed");
        let ast = Parser::new(&tokens).parse().expect("parse failed");

        assert_eq!(
            ast,
            vec![
                Node::Builtin {
                    kind: Builtin::Const,
                    lhs: "led",
                    rhs: Box::new(Node::Number(0xF)),
                },
                Node::Instruction {
                    partial: Instruction::LOADI { imm: 0 },
                    rhs: Some(Box::new(Node::Literal(Box::new(Node::Ident("led"))))),
                },
                Node::Instruction {
                    partial: Instruction::ST { addr: 0 },
                    rhs: Some(Box::new(Node::Addr(Box::new(Node::Ident("led"))))),
                },
                Node::Instruction {
                    partial: Instruction::HALT,
                    rhs: None,
                }
            ]
        );
    }

    #[test]
    fn test_fail_missing_rhs() {
        let tokens = Lexer::new("LOADI".as_bytes()).lex().expect("lex failed");
        assert!(Parser::new(&tokens).parse().is_err());
    }

    #[test]
    fn test_fail_addr_missing_bracket() {
        let tokens = Lexer::new("ST [led".as_bytes()).lex().expect("lex failed");
        assert!(Parser::new(&tokens).parse().is_err());
    }

    #[test]
    fn test_fail_builtin_invalid_rhs() {
        let tokens = Lexer::new(".const x foo".as_bytes())
            .lex()
            .expect("lex failed");
        assert!(Parser::new(&tokens).parse().is_err());
    }
}
