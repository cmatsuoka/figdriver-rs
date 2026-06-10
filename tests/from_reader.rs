use std::io::Cursor;
use std::path::PathBuf;

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn from_reader_plain_font() {
    let data = std::fs::read(manifest().join("fonts/small.flf")).unwrap();
    let font = figdriver::FIGfont::from_reader(Cursor::new(data)).unwrap();

    assert_eq!(font.height, 5);
    assert!(!font.get(65).unwrap().get().is_empty());
}

#[test]
fn from_reader_zip_font() {
    let data = std::fs::read(manifest().join("tests/fixtures/standard.flf.zip")).unwrap();
    let font = figdriver::FIGfont::from_reader(Cursor::new(data)).unwrap();

    assert_eq!(font.height, 6);
    assert!(!font.get(65).unwrap().get().is_empty());
}

#[test]
fn from_reader_file_plain() {
    let file = std::fs::File::open(manifest().join("fonts/small.flf")).unwrap();
    let font = figdriver::FIGfont::from_reader(file).unwrap();

    assert_eq!(font.height, 5);
    assert!(!font.get(65).unwrap().get().is_empty());
}

#[test]
fn from_reader_file_zip() {
    let file = std::fs::File::open(manifest().join("tests/fixtures/standard.flf.zip")).unwrap();
    let font = figdriver::FIGfont::from_reader(file).unwrap();

    assert_eq!(font.height, 6);
    assert!(!font.get(65).unwrap().get().is_empty());
}

#[test]
fn from_reader_plain_and_zip_match() {
    let plain_data = std::fs::read(manifest().join("fonts/standard.flf")).unwrap();
    let plain_font = figdriver::FIGfont::from_reader(Cursor::new(plain_data)).unwrap();

    let zip_data = std::fs::read(manifest().join("tests/fixtures/standard.flf.zip")).unwrap();
    let zip_font = figdriver::FIGfont::from_reader(Cursor::new(zip_data)).unwrap();

    assert_eq!(plain_font.height, zip_font.height);
    assert_eq!(plain_font.old_layout, zip_font.old_layout);
    assert_eq!(plain_font.layout, zip_font.layout);
}

#[test]
fn from_reader_invalid_data() {
    let data = b"not a valid font";
    let result = figdriver::FIGfont::from_reader(Cursor::new(data));
    assert!(result.is_err());
}
