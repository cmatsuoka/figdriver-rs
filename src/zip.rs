use std::fs::File;
use std::io::{BufReader, Cursor, Read};
use std::path::Path;

/// Check whether a file is a ZIP archive by reading its magic bytes.
pub fn is_zip<P: AsRef<Path>>(path: P) -> bool {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut reader = BufReader::new(&file);
    let mut header = [0u8; 2];
    reader.read_exact(&mut header).is_ok() && header == [b'P', b'K']
}

/// Decompress the first entry of a ZIP archive into a `BufReader`.
///
/// Returns a reader over the decompressed content of the archive's first
/// entry. Per the FIGfont spec, only the first entry is relevant.
pub fn decompress_zip<P: AsRef<Path>>(path: P) -> Result<BufReader<Cursor<Vec<u8>>>, std::io::Error> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut reader = archive.by_index(0)?;
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    Ok(BufReader::new(Cursor::new(buf)))
}
