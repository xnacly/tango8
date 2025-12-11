use std::fmt::{Debug, Display};

#[derive(Debug)]
pub struct T8Err {
    pub line: usize,
    pub col: usize,
    pub msg: String,
}

impl T8Err {
    pub fn render<'r, W: std::io::Write, S: Display>(
        &self,
        w: &'r mut W,
        lines: &'r [S],
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(prev) = lines.get(self.line - 2) {
            writeln!(w, "{:02} | {}", self.line - 2, prev)?;
        }
        if let Some(prev) = lines.get(self.line - 1) {
            writeln!(w, "{:02} | {}", self.line - 1, prev)?;
        }
        if let Some(cur) = lines.get(self.line) {
            writeln!(w, "{:02} | {}", self.line, cur)?;
        }
        let pad = " ".repeat(self.col - 1);
        writeln!(w, "   |{pad}^ {} ", self.msg)?;

        if let Some(after) = lines.get(self.line + 1) {
            writeln!(w, "{:02} | {}", self.line + 1, after)?;
        }
        if let Some(after) = lines.get(self.line + 2) {
            writeln!(w, "{:02} | {}", self.line + 2, after)?;
        }
        w.flush()?;
        Ok(())
    }
}
