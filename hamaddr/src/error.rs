use std::fmt;
use std::fmt::{Debug, Display, Formatter};

pub type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

/// Error type indicating an invalid character at a specific index.
#[derive(Debug, thiserror::Error)]
pub struct InvalidCharAt(pub usize);

impl Display for InvalidCharAt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

/// Error type indicating that the character was not valid.
#[derive(Debug, thiserror::Error)]
pub struct InvalidChar;

impl Display for InvalidChar {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

/// Error type indicating that the given 16-bit chunk does not decode properly.
#[derive(Debug, thiserror::Error)]
pub struct InvalidChunk;

impl Display for InvalidChunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
