use std;

#[derive(Debug)]
pub enum Error {
    Dir,
    PkDecode,
    DecodeStorage,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::Dir => write!(f, "Dir not found"),
            Error::PkDecode => write!(f, "Could not decode public key"),
            // Error::EncodeStorageKey => write!(f, "Error encoding storage key"),
            Error::DecodeStorage => write!(f, "Error decoding storage"),
        }
    }
}

impl std::error::Error for Error {}