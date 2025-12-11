use crate::lexer::{Token, TokenInner};
use shared::{asm::Instruction, err::T8Err};

#[derive(Debug, PartialEq, Eq)]
pub enum Builtin {
    Const,
}

impl TryFrom<&[u8]> for Builtin {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            b"const" => Ok(Self::Const),
            _ => Err(format!(
                "Unknown builtin `{}`",
                String::from_utf8_lossy(value)
            )),
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

macro_rules! cur {
    ($l:ident) => {
        $l.src[$l.pos]
    };
}

macro_rules! advance {
    ($l:ident) => {
        $l.pos += 1;
    };
}

impl<'parser> Parser<'parser> {
    pub fn new(src: &'parser [Token<'parser>]) -> Self {
        Parser { src, pos: 0 }
    }

    fn end(&self) -> bool {
        self.pos >= self.src.len()
    }

    fn err<S: Into<String>>(&self, msg: S) -> T8Err {
        let Token { line, col, .. } = cur!(self);
        T8Err {
            line,
            col,
            msg: msg.into(),
        }
    }

    fn parse_one(&mut self) -> Result<Node<'parser>, T8Err> {
        Ok(match cur!(self).inner {
            TokenInner::Builtin(name) => {
                let kind = (*name).try_into().map_err(|e| self.err(e))?;
                // skip .<kind>
                advance!(self);
                let lhs = if let Token {
                    inner: TokenInner::Ident(lhs),
                    ..
                } = cur!(self)
                {
                    str::from_utf8(lhs).unwrap()
                } else {
                    return Err(self.err("Wanted ident as builtin lhs, got something else"));
                };
                // skip lhs
                advance!(self);

                let rhs = match kind {
                    Builtin::Const => match cur!(self).inner {
                        TokenInner::Number(n) => Node::Number(n),
                        _ => {
                            return Err(self.err("Invalid rhs for .const, wanted number"));
                        }
                    },
                };

                // skip argument
                advance!(self);

                Node::Builtin {
                    kind,
                    lhs,
                    rhs: Box::new(rhs),
                }
            }
            TokenInner::Ident(ident) => {
                let partial = Instruction::from_str_lossy(str::from_utf8(ident).unwrap())
                    .map_err(|e| self.err(e))?;
                // skip self
                advance!(self);
                let rhs = match &partial {
                    Instruction::LOADI { .. }
                    | Instruction::ST { .. }
                    | Instruction::ROL { .. } => Some(Box::new(self.parse_one()?)),
                    _ => None,
                };
                Node::Instruction { partial, rhs }
            }
            TokenInner::Hash => {
                // skip #
                advance!(self);
                let inner = match cur!(self).inner {
                    TokenInner::Number(n) => Node::Number(n),
                    TokenInner::Ident(ident) => Node::Ident(str::from_utf8(ident).unwrap()),
                    _ => {
                        return Err(self.err("Invalid inner literal, wanted ident or number"));
                    }
                };
                // skip number or ident
                advance!(self);
                Node::Literal(Box::new(inner))
            }
            TokenInner::LeftBraket => {
                // skip [
                advance!(self);
                let inner = match cur!(self).inner {
                    TokenInner::Number(n) => Node::Number(n),
                    TokenInner::Ident(ident) => Node::Ident(str::from_utf8(ident).unwrap()),
                    _ => {
                        return Err(self.err("Invalid inner addr, wanted ident or number"));
                    }
                };
                // skip inner
                advance!(self);

                let addr = Node::Addr(Box::new(inner));
                if cur!(self).inner != TokenInner::RightBraket {
                    return Err(self.err("] needed for addr syntax"));
                }

                // skip ]
                advance!(self);
                addr
            }
            TokenInner::Number(n) => {
                advance!(self);
                Node::Number(n)
            }
            _ => return Err(self.err(format!("Unkown token type {:?}", cur!(self)))),
        })
    }

    pub fn parse(&mut self) -> Result<Vec<Node<'parser>>, T8Err> {
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
