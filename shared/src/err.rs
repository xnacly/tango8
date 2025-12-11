use std::fmt::Debug;

#[derive(Debug)]
pub struct T8Err {
    pub line: usize,
    pub col: usize,
    pub msg: String,
}

impl T8Err {
    pub fn render<'r, W, S>(
        &self,
        w: &'r mut W,
        lines: &'r [S],
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        W: std::io::Write,
        S: std::fmt::Display,
    {
        let start = self.line.saturating_sub(2);
        let end = (self.line + 3).min(lines.len());

        for i in start..end {
            writeln!(w, "{:02} | {}", i + 1, &lines[i])?;
            if i == self.line {
                let pad = " ".repeat(self.col.saturating_sub(1));
                writeln!(w, "   |{pad}^ {}", self.msg)?;
            }
        }

        w.flush()?;
        Ok(())
    }
}
