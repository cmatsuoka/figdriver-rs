use std::error;
use std::fmt;
use std::io;
use std::num;
use std::path::PathBuf;

pub use self::figfont::*;
pub use self::wrapper::{Align, Wrapper};
pub use self::smusher::Smusher;

mod figfont;
mod wrapper;
mod smusher;

#[derive(Debug)]
pub enum Error {
    FontFormat(&'static str),
    Io(io::Error),
    Parse(num::ParseIntError),
    CodeTag(u32),
    LineFull,
    FontNotFound(PathBuf),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::FontFormat(descr) => write!(f, "{}", descr),
            Error::Io(err)           => write!(f, "{}", err),
            Error::Parse(err)        => write!(f, "Can't parse value: {}", err),
            Error::CodeTag(tag)      => write!(f, "Invalid code tag: {}", tag),
            Error::LineFull          => write!(f, "Line is full"),
            Error::FontNotFound(path) => write!(f, "Font not found: {}", path.display()),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Io(err)         => Some(err),
            Error::Parse(err)      => Some(err),
            Error::FontNotFound(_) => None,
            _                      => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(err: num::ParseIntError) -> Error {
        Error::Parse(err)
    }
}
