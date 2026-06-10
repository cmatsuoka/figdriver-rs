//! FIGdriver-rs is a Rust library for rendering text in large ASCII-art characters using
//! FIGfont format fonts, compatible with the [FIGlet](https://www.figlet.org/) specification.
//!
//! The library offers two ways to render text:
//!
//! - **`Wrapper`** — High-level API with automatic word wrapping and alignment. Use
//!   `write_line()` for single-line rendering or `write_paragraph()` for multi-line
//!   paragraph mode. Accepts string input and flushes rendered lines via a callback.
//! - **`Smusher`** — Lower-level API for direct character composition. Accepts strings
//!   (`push_str`), single characters (`push`), or raw codes (`push_codes`). Create with
//!   `Smusher::new()` for defaults or `Smusher::builder()` for full control over layout
//!   mode, direction, and control pipelines.
//!
//! # Quick Start
//!
//! ```
//! use figdriver::{FIGfont, Smusher};
//!
//! # fn foo() -> Result<(), Box<dyn std::error::Error>> {
//! let font = FIGfont::from_path("fonts/small.flf")?;
//! let mut sm = Smusher::new(&font);
//! sm.push_str("Hi!");
//!
//! for line in sm.get() {
//!     println!("{}", line);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Using the Wrapper
//!
//! The `Wrapper` handles word wrapping, alignment, and output formatting on top of a `Smusher`.
//!
//! ```
//! use figdriver::{FIGfont, Smusher, Wrapper, Control, Align};
//!
//! # fn foo() -> Result<(), Box<dyn std::error::Error>> {
//! let font = FIGfont::from_path("fonts/small.flf")?;
//! let sm = Smusher::new(&font);
//! let mut wr = Wrapper::new(sm, Control::default(), 80, Align::Left);
//!
//! wr.write_line("Hello, world!", &|lines: &[String]| {
//!     for line in lines {
//!         println!("{}", line);
//!     }
//! });
//! # Ok(())
//! # }
//! ```
//!
//! # Using the Smusher Directly
//!
//! For fine-grained control, use `Smusher` with the builder to adjust layout mode or other
//! settings.
//!
//! ```
//! use figdriver::{FIGfont, Smusher, LayoutMode};
//!
//! # fn foo() -> Result<(), Box<dyn std::error::Error>> {
//! let font = FIGfont::from_path("fonts/small.flf")?;
//! let mut sm = Smusher::builder(&font)
//!     .layout_mode(LayoutMode::FullWidth)
//!     .build();
//! sm.push_str("Hi!");
//! # Ok(())
//! # }
//! ```
//!
//! # Key Types
//!
//! - [`FIGfont`] — Parsed FIGfont (.flf) file, the font data source.
//! - [`FIGchar`] — A single multi-line character definition from a font.
//! - [`Smusher`] — Composes characters into a multi-line output buffer with smushing.
//! - [`Wrapper`] — Adds word wrapping, alignment, and paragraph mode on top of a Smusher.
//! - [`Control`] — Character-mapping pipeline (fnc files) for ligatures and substitutions.
//! - [`Error`] — Error type covering font loading, I/O, parsing, and rendering failures.

use std::error;
use std::fmt;
use std::io;
use std::num;
use std::path::PathBuf;

pub use self::figfont::{
    FIGchar, FIGfont,
    SMUSH_ENABLE, SMUSH_EQUAL, SMUSH_HARDBLANK, SMUSH_HIERARCHY,
    SMUSH_KERN, SMUSH_PAIR, SMUSH_UNDERLINE, SMUSH_BIGX,
};
pub use self::wrapper::{Align, Wrapper, is_whitespace_code};
pub use self::smusher::{LayoutMode, Smusher, SmusherBuilder};
pub use self::control::{Control, Flc, InputEncoding};

mod control;
mod figfont;
mod wrapper;
mod smusher;
mod zip;

/// Errors produced by figdriver operations.
///
/// This enum covers font loading, I/O, parsing, control file handling,
/// and wrapper rendering errors.
#[derive(Debug)]
pub enum Error {
    /// Malformed or unsupported font file format.
    FontFormat(&'static str),
    /// I/O error from reading a file.
    Io(io::Error),
    /// Integer parsing error from a font header value.
    Parse(num::ParseIntError),
    /// Invalid FIGchar code tag value.
    CodeTag(i32),
    /// The wrapper's output line is full and cannot accept more characters.
    LineFull,
    /// Font file was not found at the given path.
    FontNotFound(PathBuf),
    /// Malformed or unsupported control file format.
    ControlFormat(&'static str),
    /// A control file mapping has mismatched range sizes.
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
