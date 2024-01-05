use std::io::{BufReader, Read};
use anyhow::Result;

pub fn read_file<S: AsRef<str>>(file_name: S) -> Result<Vec<u8>> {
    let file = std::fs::File::open(file_name.as_ref())?;
    let mut reader = BufReader::new(file);
    let mut contents = Vec::new();
    reader.read_to_end(&mut contents)?;
    Ok(contents)
}