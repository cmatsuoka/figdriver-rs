use std::error;
use std::fmt;
use std::io;
use std::num;
use std::path::PathBuf;

pub use self::figfont::*;
pub use self::wrapper::{Align, Wrapper};
pub use self::smusher::{LayoutMode, Smusher, SmusherBuilder};
pub use self::control::{Control, EncodingDecoder, Flc, InputEncoding};

mod control;
mod figfont;
mod wrapper;
mod smusher;
mod zip;

#[derive(Debug)]
pub enum Error {
    FontFormat(&'static str),
    Io(io::Error),
    Parse(num::ParseIntError),
    CodeTag(i32),
    LineFull,
    FontNotFound(PathBuf),
    ControlFormat(&'static str),
    ControlRangeMismatch,
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
            Error::ControlFormat(msg) => write!(f, "Malformed control file: {}", msg),
            Error::ControlRangeMismatch => write!(f, "Range sizes do not match"),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Io(err)              => Some(err),
            Error::Parse(err)           => Some(err),
            Error::FontNotFound(_)      => None,
            Error::ControlFormat(_)     => None,
            Error::ControlRangeMismatch => None,
            _                           => None,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as _;

    #[test]
    fn test_error_display() {
        assert_eq!(format!("{}", Error::FontFormat("bad header")), "bad header");
        assert_eq!(format!("{}", Error::CodeTag(0x12345)), "Invalid code tag: 74565");
        assert_eq!(format!("{}", Error::LineFull), "Line is full");
        assert_eq!(
            format!("{}", Error::FontNotFound(PathBuf::from("/foo/bar.flf"))),
            "Font not found: /foo/bar.flf"
        );
    }

    #[test]
    fn test_error_display_parse() {
        let err: Error = "not_a_number".parse::<i32>().unwrap_err().into();
        assert!(format!("{}", err).starts_with("Can't parse value:"));
    }

    #[test]
    fn test_error_display_io() {
        let err: Error = Error::Io(io::Error::new(io::ErrorKind::NotFound, "no such file"));
        assert_eq!(format!("{}", err), "no such file");
    }

    #[test]
    fn test_error_source() {
        assert!(Error::FontFormat("x").source().is_none());
        assert!(Error::CodeTag(1).source().is_none());
        assert!(Error::LineFull.source().is_none());
        assert!(Error::FontNotFound(PathBuf::from("/x")).source().is_none());
        assert!(Error::ControlFormat("x").source().is_none());
        assert!(Error::ControlRangeMismatch.source().is_none());

        let io_err = io::Error::new(io::ErrorKind::NotFound, "msg");
        assert!(Error::Io(io_err).source().is_some());

        let parse_err: num::ParseIntError = "x".parse::<i32>().unwrap_err();
        assert!(Error::Parse(parse_err).source().is_some());
    }
}
