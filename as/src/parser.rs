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
            _ => Err("Invalid builtin".into()),
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
    Label {
        name: &'node str,
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
    Ident {
        pos: (usize, usize),
        inner: &'node str,
    },
}

pub struct Parser<'parser> {
    src: &'parser [Token<'parser>],
    pos: usize,
}

macro_rules! cur {
    ($l:ident) => {
        $l.src.get($l.pos).ok_or_else(|| T8Err {
            line: 0,
            col: 0,
            msg: "Unexpected End of File".into(),
        })?
    };
}

macro_rules! next {
    ($l:ident) => {
        $l.src.get($l.pos + 1)
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
        let Token { line, col, .. } = self.src.get(self.pos).unwrap_or_else(|| &Token {
            line: 0,
            col: 0,
            inner: TokenInner::Eof,
        });
        T8Err {
            line: *line,
            col: *col,
            msg: msg.into(),
        }
    }

    fn parse_rhs(&mut self) -> Result<Node<'parser>, T8Err> {
        let Token { inner, line, col } = cur!(self);
        Ok(match inner {
            TokenInner::Ident(ident) => {
                advance!(self);
                Node::Ident {
                    pos: (*line, *col),
                    inner: str::from_utf8(ident).unwrap(),
                }
            }
            TokenInner::Hash => {
                // skip #
                advance!(self);
                let inner = match cur!(self).inner {
                    TokenInner::Number(n) => Node::Number(n),
                    _ => {
                        return Err(self.err("Invalid inner literal, wanted number"));
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
                    TokenInner::Ident(ident) => Node::Ident {
                        pos: (*line, *col),
                        inner: str::from_utf8(ident).unwrap(),
                    },
                    _ => {
                        return Err(self.err("Invalid inner addr, wanted ident or number"));
                    }
                };

                let Token { line, col, .. } = cur!(self);

                // skip inner
                advance!(self);

                let addr = Node::Addr(Box::new(inner));
                if self.src.get(self.pos)
                    != Some(&Token {
                        inner: TokenInner::RightBraket,
                        line: *line,
                        col: *col,
                    })
                {
                    return Err({
                        T8Err {
                            line: *line,
                            col: *col,
                            msg: "`]` postfix needed for addr syntax".into(),
                        }
                    });
                }

                // skip ]
                advance!(self);
                addr
            }
            TokenInner::Number(n) => {
                advance!(self);
                Node::Number(*n)
            }
            _ => {
                return Err(self.err(format!(
                    "Unexpected token `{:?}` at this point",
                    cur!(self).inner
                )));
            }
        })
    }

    fn parse_one(&mut self) -> Result<Node<'parser>, T8Err> {
        if self.end() {
            let (line, col) = self
                .src
                .last()
                .map(|l| (l.line, l.col))
                .unwrap_or_else(|| (0, 0));

            return Err(T8Err {
                line,
                col,
                msg: "Unexpected end of input".into(),
            });
        }
        let Token { inner, .. } = cur!(self);
        Ok(match inner {
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
            // <label>:
            TokenInner::Ident(ident)
                if matches!(
                    next!(self),
                    Some(Token {
                        inner: TokenInner::Colon,
                        ..
                    })
                ) =>
            {
                // skip ident
                advance!(self);
                // skip :
                advance!(self);
                Node::Label {
                    name: str::from_utf8(ident).unwrap(),
                }
            }
            TokenInner::Ident(ident) => {
                let partial = Instruction::from_str_lossy(str::from_utf8(ident).unwrap())
                    .map_err(|e| self.err(e))?;

                // skip self
                advance!(self);

                let rhs = match &partial {
                    Instruction::LOADI { .. } | Instruction::ST { .. } | Instruction::LD { .. } => {
                        if self.pos >= self.src.len() {
                            return Err(self.err("Missing rhs"));
                        }
                        Some(Box::new(self.parse_rhs()?))
                    }
                    // explicitly None so the compiler wont let me skip this when adding new ones
                    Instruction::NOP
                    | Instruction::JMP
                    | Instruction::ROL1
                    | Instruction::MOV
                    | Instruction::ADD
                    | Instruction::SUB
                    | Instruction::HALT => None,
                };
                Node::Instruction { partial, rhs }
            }
            _ => {
                return Err(self.err(format!(
                    "Unkown token type `{:?}` at this point",
                    cur!(self)
                )));
            }
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
        let tokens = Lexer::new(".const foo 0xF\nLOADI foo".as_bytes())
            .lex()
            .expect("lex failed");
        let ast = Parser::new(&tokens).parse().expect("parse failed");

        assert_eq!(
            ast,
            vec![
                Node::Builtin {
                    kind: Builtin::Const,
                    lhs: "foo",
                    rhs: Box::new(Node::Number(15))
                },
                Node::Instruction {
                    partial: Instruction::LOADI { imm: 0 },
                    rhs: Some(Box::new(Node::Ident {
                        pos: (1, 10),
                        inner: "foo"
                    }))
                }
            ]
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
                rhs: Some(Box::new(Node::Addr(Box::new(Node::Ident {
                    pos: (0, 3),
                    inner: "led"
                })))),
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
LOADI led
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
                    rhs: Box::new(Node::Number(15))
                },
                Node::Instruction {
                    partial: Instruction::LOADI { imm: 0 },
                    rhs: Some(Box::new(Node::Ident {
                        pos: (2, 10),
                        inner: "led"
                    }))
                },
                Node::Instruction {
                    partial: Instruction::ST { addr: 0 },
                    rhs: Some(Box::new(Node::Addr(Box::new(Node::Ident {
                        pos: (3, 4),
                        inner: "led"
                    }))))
                },
                Node::Instruction {
                    partial: Instruction::HALT,
                    rhs: None
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
