use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;

/// Maximum allowed decompressed size for a ZIP entry (10 MB).
const MAX_DECOMPRESSED_SIZE: u64 = 10 * 1024 * 1024;

/// Check whether a file is a ZIP archive by reading its magic bytes.
pub fn is_zip<P: AsRef<Path>>(path: P) -> bool {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut header = [0u8; 2];
    file.read_exact(&mut header).is_ok() && header == [b'P', b'K']
}

/// Decompress the first entry of a ZIP archive into a `Cursor`.
///
/// Returns a reader over the decompressed content of the archive's first
/// entry. Per the FIGfont spec, only the first entry is relevant.
/// Decompressed size is capped at `MAX_DECOMPRESSED_SIZE` to prevent
/// zip bomb attacks.
pub fn decompress_zip<P: AsRef<Path>>(path: P) -> Result<Cursor<Vec<u8>>, std::io::Error> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let reader = archive.by_index(0)?;
    let mut buf = Vec::new();
    reader.take(MAX_DECOMPRESSED_SIZE).read_to_end(&mut buf)?;
    Ok(Cursor::new(buf))
}
