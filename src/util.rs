use std::io::{BufReader, Read};
use base64::engine::general_purpose;
use base64::{Engine};

pub fn read_file<S: AsRef<str>>(file_name: S) -> Result<Vec<u8>, std::io::Error> {
    let file = std::fs::File::open(file_name.as_ref())?;
    let mut reader = BufReader::new(file);
    let mut contents = Vec::new();
    reader.read_to_end(&mut contents)?;
    Ok(contents)
}

pub fn base64_encode<S>(content: S) -> String
    where
        S: AsRef<[u8]>,
{
    general_purpose::STANDARD.encode(content)
}
