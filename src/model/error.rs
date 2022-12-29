use core::fmt::{Display, Formatter, Result};
use std::error;

#[derive(Debug)]
pub struct Cause {
    cause: String,
}

impl From<&str> for Cause {
    fn from(cause: &str) -> Self {
        Cause {
            cause: String::from(cause),
        }
    }
}

impl Display for Cause {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.cause)
    }
}

impl error::Error for Cause {}

#[derive(Debug)]
pub enum Error {
    Wallet(Box<dyn error::Error>),
    Codec(Box<dyn error::Error>),
    Sube(Box<dyn error::Error>),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Error::Wallet(reason) => write!(f, "Error handling wallet: {}", reason),
            Error::Codec(reason) => write!(f, "Error encoding/decoding data: {}", reason),
            Error::Sube(reason) => write!(f, "Error handling connection with sube: {:?}", reason),
        }
    }
}

impl error::Error for Error {}
