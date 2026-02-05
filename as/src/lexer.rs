use core::fmt;

use shared::err::T8Err;

pub struct Lexer<'lex> {
    src: &'lex [u8],
    pos: usize,
    line: usize,
    col: usize,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Token<'tok> {
    pub line: usize,
    pub col: usize,
    pub inner: TokenInner<'tok>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TokenInner<'tok> {
    Eof,
    Ident(&'tok [u8]),
    Builtin(&'tok [u8]),
    Hash,
    LeftBraket,
    RightBraket,
    Colon,
    Comma,
    Number(&'tok [u8]),
}

impl<'tok> fmt::Debug for TokenInner<'tok> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenInner::Eof => write!(f, "EOF"),
            TokenInner::Ident(name) => write!(f, "Ident({})", String::from_utf8_lossy(name)),
            TokenInner::Builtin(name) => write!(f, "Builtin({})", String::from_utf8_lossy(name)),
            TokenInner::Hash => write!(f, "Hash"),
            TokenInner::LeftBraket => write!(f, "LeftBracket"),
            TokenInner::RightBraket => write!(f, "RightBracket"),
            TokenInner::Number(n) => write!(f, "Number({})", String::from_utf8_lossy(n)),
            TokenInner::Colon => write!(f, ":"),
            TokenInner::Comma => write!(f, ","),
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

    fn tok(&self, inner: TokenInner<'lex>) -> Token<'lex> {
        Token {
            line: self.line,
            col: self.col,
            inner,
        }
    }

    fn err<S: Into<String>>(&self, msg: S) -> T8Err {
        T8Err {
            line: self.line,
            col: self.col,
            msg: msg.into(),
        }
    }

    fn end(&self) -> bool {
        self.cur().is_none()
    }

    fn cur(&self) -> Option<&u8> {
        self.src.get(self.pos)
    }

    fn advance(&mut self) {
        self.pos += 1;
        self.col += 1
    }

    pub fn lex(&mut self) -> Result<Vec<Token<'lex>>, T8Err> {
        let mut toks = vec![];
        while !self.end() {
            let Some(c) = self.cur() else {
                break;
            };

            match *c as char {
                ';' => {
                    while self.cur().is_some_and(|b| *b != b'\n') {
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
                    if !self.cur().is_some_and(|b| b.is_ascii_alphabetic()) {
                        return Err(self.err("A '.' requires a following builtin name"));
                    }
                    let start = self.pos;
                    while self.cur().is_some_and(|b| b.is_ascii_alphabetic()) {
                        self.advance()
                    }
                    toks.push(self.tok(TokenInner::Builtin(&self.src[start..self.pos])))
                }
                ',' => {
                    toks.push(self.tok(TokenInner::Comma));
                    self.advance()
                }
                '#' => {
                    toks.push(self.tok(TokenInner::Hash));
                    self.advance()
                }
                ':' => {
                    toks.push(self.tok(TokenInner::Colon));
                    self.advance()
                }
                '[' => {
                    toks.push(self.tok(TokenInner::LeftBraket));
                    self.advance()
                }

                ']' => {
                    toks.push(self.tok(TokenInner::RightBraket));
                    self.advance()
                }
                '0'..='9' => {
                    let start = self.pos;
                    while self
                        .cur()
                        .is_some_and(|b| matches!(b, b'x' | b'0'..=b'9' | b'A'..=b'F'))
                    {
                        self.advance()
                    }
                    toks.push(self.tok(TokenInner::Number(&self.src[start..self.pos])))
                }
                'a'..='z' | 'A'..='Z' => {
                    let start = self.pos;
                    while self
                        .cur()
                        .is_some_and(|b| b.is_ascii_alphanumeric() || b == &b'_')
                    {
                        self.advance()
                    }
                    toks.push(self.tok(TokenInner::Ident(&self.src[start..self.pos])))
                }
                _ => {
                    return Err(self.err(format!(
                        "Unkown character `{}`",
                        self.cur().map(|b| *b as char).unwrap(),
                    )));
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
    LOADI off
    LD [led]
    ST [led]
    LOADI on
    ST [led]
    

; 8 leds mapped at 'led'
    LOADI #0xD  ; 0b00001101
    ST [led]    ; LEDs 0,2,3 on
"#;

        let mut lexer = Lexer::new(src);
        let tokens = lexer.lex().expect("Lexer failed");

        let expected = vec![
            TokenInner::Builtin(b"const"),
            TokenInner::Ident(b"led"),
            TokenInner::Number(b"0xF"),
            TokenInner::Builtin(b"const"),
            TokenInner::Ident(b"off"),
            TokenInner::Number(b"0"),
            TokenInner::Builtin(b"const"),
            TokenInner::Ident(b"on"),
            TokenInner::Number(b"1"),
            TokenInner::Ident(b"LOADI"),
            TokenInner::Ident(b"off"),
            TokenInner::Ident(b"LD"),
            TokenInner::LeftBraket,
            TokenInner::Ident(b"led"),
            TokenInner::RightBraket,
            TokenInner::Ident(b"ST"),
            TokenInner::LeftBraket,
            TokenInner::Ident(b"led"),
            TokenInner::RightBraket,
            TokenInner::Ident(b"LOADI"),
            TokenInner::Ident(b"on"),
            TokenInner::Ident(b"ST"),
            TokenInner::LeftBraket,
            TokenInner::Ident(b"led"),
            TokenInner::RightBraket,
            TokenInner::Ident(b"LOADI"),
            TokenInner::Hash,
            TokenInner::Number(b"0xD"),
            TokenInner::Ident(b"ST"),
            TokenInner::LeftBraket,
            TokenInner::Ident(b"led"),
            TokenInner::RightBraket,
        ];

        assert_eq!(
            tokens.len(),
            expected.len(),
            "Token count mismatch {:?}",
            tokens
        );

        for (i, (got, exp)) in tokens.iter().zip(expected.iter()).enumerate() {
            assert_eq!(
                got.inner, *exp,
                "Mismatch at token {}: got {:?}, expected {:?}",
                i, got, exp
            );
        }
    }

    #[test]
    fn test_basic_tokens() {
        let src = br#".const #label [ ] : , 123ABC"#;
        let mut lexer = Lexer::new(src);
        let tokens = lexer.lex().expect("Lexer failed");

        let expected = vec![
            TokenInner::Builtin(b"const"),
            TokenInner::Hash,
            TokenInner::Ident(b"label"),
            TokenInner::LeftBraket,
            TokenInner::RightBraket,
            TokenInner::Colon,
            TokenInner::Comma,
            TokenInner::Number(b"123ABC"),
        ];

        assert_eq!(tokens.len(), expected.len(), "Token count mismatch");

        for (i, (tok, exp)) in tokens.iter().zip(expected.iter()).enumerate() {
            assert_eq!(
                tok.inner, *exp,
                "Token mismatch at {}: got {:?}, expected {:?}",
                i, tok.inner, exp
            );
        }
    }

    #[test]
    fn test_whitespace_and_comments() {
        let src = br#"
            ; comment line
            LOADI #0 ; inline comment
            ST [led]
            "#;
        let mut lexer = Lexer::new(src);
        let tokens = lexer.lex().expect("Lexer failed");

        let expected = vec![
            TokenInner::Ident(b"LOADI"),
            TokenInner::Hash,
            TokenInner::Number(b"0"),
            TokenInner::Ident(b"ST"),
            TokenInner::LeftBraket,
            TokenInner::Ident(b"led"),
            TokenInner::RightBraket,
        ];

        assert_eq!(tokens.len(), expected.len(), "Token count mismatch");

        for (i, (tok, exp)) in tokens.iter().zip(expected.iter()).enumerate() {
            assert_eq!(
                tok.inner, *exp,
                "Token mismatch at {}: got {:?}, expected {:?}",
                i, tok.inner, exp
            );
        }
    }

    #[test]
    fn test_error_on_invalid_builtin() {
        let src = br#".1invalid"#;
        let mut lexer = Lexer::new(src);
        let err = lexer.lex().unwrap_err();
        assert!(err.msg.contains("A '.' requires a following builtin name"));
    }

    #[test]
    fn test_error_on_unknown_char() {
        let src = br#"@"#;
        let mut lexer = Lexer::new(src);
        let err = lexer.lex().unwrap_err();
        assert!(err.msg.contains("Unkown character"));
    }

    #[test]
    fn test_numbers_hex_and_decimal() {
        let src = br#"0 1 12 0xA 0xFF"#;
        let mut lexer = Lexer::new(src);
        let tokens = lexer.lex().expect("Lexer failed");

        let expected = vec![
            TokenInner::Number(b"0"),
            TokenInner::Number(b"1"),
            TokenInner::Number(b"12"),
            TokenInner::Number(b"0xA"),
            TokenInner::Number(b"0xFF"),
        ];

        assert_eq!(tokens.len(), expected.len());

        for (i, (tok, exp)) in tokens.iter().zip(expected.iter()).enumerate() {
            assert_eq!(tok.inner, *exp, "Mismatch at {}: got {:?}", i, tok.inner);
        }
    }

    #[test]
    fn test_ident_and_builtins() {
        let src = br#".const myLabel anotherLabel"#;
        let mut lexer = Lexer::new(src);
        let tokens = lexer.lex().expect("Lexer failed");

        let expected = vec![
            TokenInner::Builtin(b"const"),
            TokenInner::Ident(b"myLabel"),
            TokenInner::Ident(b"anotherLabel"),
        ];

        assert_eq!(tokens.len(), expected.len());

        for (i, (tok, exp)) in tokens.iter().zip(expected.iter()).enumerate() {
            assert_eq!(tok.inner, *exp, "Mismatch at {}: got {:?}", i, tok.inner);
        }
    }
}
