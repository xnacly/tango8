use t8::asm::Instruction;

use crate::lexer::Token;

#[derive(Debug)]
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

#[derive(Debug)]
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
    Number(usize),
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

    fn end(&mut self) -> bool {
        self.pos >= self.src.len()
    }

    fn cur(&mut self) -> &Token<'parser> {
        &self.src[self.pos]
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn parse_one(&mut self) -> Result<Node<'parser>, String> {
        Ok(match *self.cur() {
            Token::Builtin(name) => {
                // skip .<kind>
                self.advance();
                let lhs = if let Token::Ident(lhs) = self.cur() {
                    str::from_utf8(lhs).unwrap()
                } else {
                    return Err("Wanted ident as builtin lhs, got something else".into());
                };
                // skip lhs
                self.advance();

                Node::Builtin {
                    kind: name.try_into()?,
                    lhs,
                    rhs: Box::new(self.parse_one()?),
                }
            }
            Token::Ident(ident) => {
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
            Token::Hash => {
                // skip #
                self.advance();
                let inner = match self.cur() {
                    Token::Number(n) => Node::Number(*n),
                    Token::Ident(ident) => Node::Ident(str::from_utf8(ident).unwrap()),
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
            Token::LeftBraket => {
                // skip [
                self.advance();
                let inner = match self.cur() {
                    Token::Number(n) => Node::Number(*n),
                    Token::Ident(ident) => Node::Ident(str::from_utf8(ident).unwrap()),
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
                if self.cur() != &Token::RightBraket {
                    return Err("] needed for addr syntax".into());
                }

                // skip ]
                self.advance();
                addr
            }
            Token::Number(n) => {
                self.advance();
                Node::Number(n)
            }
            _ => return Err(format!("Unkown token type {:?}", self.cur())),
        })
    }

    pub fn parse(&mut self) -> Result<Vec<Node<'parser>>, String> {
        let mut r = vec![];
        while !self.end() {
            r.push(dbg!(self.parse_one()?));
        }
        Ok(r)
    }
}
