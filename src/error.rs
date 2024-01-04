use strum_macros::Display;

#[derive(Debug, Display)]
pub enum Error {
    Io(String),
}
