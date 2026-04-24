use core::fmt::Display;

use alloc::string::String;

use crate::Value;

/// An error that occurred during JavaScript execution.
#[derive(Debug)]
pub struct Error {
    pub(crate) message: String,
    pub(crate) exception: Value,
}

impl Error {
    /// The error message of the exception.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The exception object.
    pub fn exception(&self) -> Value {
        self.exception
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl core::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;
