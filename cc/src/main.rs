use std::{fs, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = std::env::args()
        .nth(1)
        .ok_or_else(|| "Missing .lisp file".to_string())?;
    let bytes = fs::read(&input)?;
    let mut path = Path::new(&input).to_path_buf();
    path.set_extension("t8b");
    fs::write(path, &bytes)?;

    Ok(())
}
