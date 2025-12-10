use core::fmt;

pub struct Lexer<'lex> {
    src: &'lex [u8],
    pos: usize,
    line: usize,
    col: usize,
}

#[derive(PartialEq, Eq)]
pub enum Token<'tok> {
    Ident(&'tok [u8]),
    Builtin(&'tok [u8]),
    Hash,
    LeftBraket,
    RightBraket,
    Number(usize),
}

impl<'tok> fmt::Debug for Token<'tok> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Ident(name) => write!(f, "Ident({})", String::from_utf8_lossy(name)),
            Token::Builtin(name) => write!(f, "Builtin({})", String::from_utf8_lossy(name)),
            Token::Hash => write!(f, "Hash"),
            Token::LeftBraket => write!(f, "LeftBracket"),
            Token::RightBraket => write!(f, "RightBracket"),
            Token::Number(n) => write!(f, "Number({})", n),
        }
    }
}

impl<'lex> Lexer<'lex> {
    pub fn new(src: &'lex [u8]) -> Lexer<'lex> {
        Lexer {
            src,
            line: 0,
            pos: 0,
            col: 0,
        }
    }

    fn end(&self) -> bool {
        self.pos >= self.src.len()
    }

    fn cur(&self) -> u8 {
        self.src[self.pos]
    }

    fn advance(&mut self) {
        self.pos += 1;
        self.col += 1
    }

    pub fn lex(&mut self) -> Result<Vec<Token<'lex>>, String> {
        let mut toks = vec![];
        while !self.end() {
            match self.cur() as char {
                ';' => {
                    while self.cur() != b'\n' {
                        self.advance();
                    }
                }
                '\n' | '\r' => {
                    self.line += 1;
                    self.col = 0;

                    self.advance();
                }
                ' ' => self.advance(),
                '.' => {
                    self.advance();
                    if !self.cur().is_ascii_alphabetic() {
                        return Err("A '.' requires a following builtin name".into());
                    }
                    let start = self.pos;
                    while self.cur().is_ascii_alphabetic() {
                        self.advance()
                    }
                    toks.push(Token::Builtin(&self.src[start..self.pos]))
                }
                '#' => {
                    toks.push(Token::Hash);
                    self.advance()
                }
                '[' => {
                    toks.push(Token::LeftBraket);
                    self.advance()
                }

                ']' => {
                    toks.push(Token::RightBraket);
                    self.advance()
                }
                '0'..='9' => {
                    let start = self.pos;
                    while !self.cur().is_ascii_whitespace() {
                        self.advance()
                    }
                    let view = &self.src[start..self.pos];
                    let as_str = str::from_utf8(view).expect("Wtf");
                    let i = if view.get(1).is_some_and(|e| *e == b'x') {
                        usize::from_str_radix(&as_str[2..as_str.len()], 16)
                    } else {
                        as_str.parse()
                    }
                    .map_err(|e| format!("{e}: `{as_str}`"))?;
                    toks.push(Token::Number(i))
                }
                'a'..='z' | 'A'..='Z' => {
                    let start = self.pos;
                    while self.cur().is_ascii_alphabetic() {
                        self.advance()
                    }
                    toks.push(Token::Ident(&self.src[start..self.pos]))
                }
                _ => {
                    return Err(format!(
                        "Unkown character {:?} {}:{}",
                        self.cur() as char,
                        self.line,
                        self.col
                    ));
                }
            }
        }
        Ok(toks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_sequence() {
        let src = br#"
; vim: filetype=asm
; 
; simple example of blinking an io mapped led, either single or 8bit addressed
; via 1 byte led array. 
;
; Assemble via: cargo run -p t8asm examples/led.t8
; Emulate via: cargo run -p t8emu examples/led.t8b

.const led 0xF
.const off 0
.const on 1

; simple on/off LED
    LOADI #off
    ST [led]
    LOADI #on
    ST [led]

; 8 leds mapped at 'led'
    LOADI #0xD  ; 0b00001101
    ST [led]    ; LEDs 0,2,3 on
"#;

        let mut lexer = Lexer::new(src);
        let tokens = lexer.lex().expect("Lexer failed");

        let expected = vec![
            Token::Builtin(b"const"),
            Token::Ident(b"led"),
            Token::Number(0xF),
            Token::Builtin(b"const"),
            Token::Ident(b"off"),
            Token::Number(0),
            Token::Builtin(b"const"),
            Token::Ident(b"on"),
            Token::Number(1),
            Token::Ident(b"LOADI"),
            Token::Hash,
            Token::Ident(b"off"),
            Token::Ident(b"ST"),
            Token::LeftBraket,
            Token::Ident(b"led"),
            Token::RightBraket,
            Token::Ident(b"LOADI"),
            Token::Hash,
            Token::Ident(b"on"),
            Token::Ident(b"ST"),
            Token::LeftBraket,
            Token::Ident(b"led"),
            Token::RightBraket,
            Token::Ident(b"LOADI"),
            Token::Hash,
            Token::Number(0xD),
            Token::Ident(b"ST"),
            Token::LeftBraket,
            Token::Ident(b"led"),
            Token::RightBraket,
        ];

        assert_eq!(
            tokens.len(),
            expected.len(),
            "Token count mismatch {:?}",
            tokens
        );

        for (i, (got, exp)) in tokens.iter().zip(expected.iter()).enumerate() {
            assert_eq!(
                got, exp,
                "Mismatch at token {}: got {:?}, expected {:?}",
                i, got, exp
            );
        }
    }
}
