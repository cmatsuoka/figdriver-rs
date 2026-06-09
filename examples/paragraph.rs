use std::io::{self, BufRead};
use figdriver::{Align, Control, FIGfont, Smusher, Wrapper};

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => eprintln!("Error: {}", e),
    }
}

// This example demonstrates the new Wrapper API for line and paragraph mode
// rendering. It reads text from stdin and renders it using the small FIGfont,
// using paragraph mode to join lines with spaces and flush on blank lines.

fn run() -> Result<(), figdriver::Error> {
    let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/fonts/small.flf";
    let font = FIGfont::from_path(&path)?;
    let control = Control::default();

    let mut wr = Wrapper::new(Smusher::new(&font), control, 78, Align::Left);

    let print_fn = |lines: &[String]| {
        for line in lines {
            println!("{}", line);
        }
    };

    // Read input from stdin and render in paragraph mode.
    // Paragraph mode joins consecutive non-blank lines with spaces,
    // and flushes on blank lines or lines starting with a space.
    // Trailing newlines are handled internally by write_paragraph.
    let mut input = io::BufReader::new(io::stdin());
    let mut buf = Vec::new();

    loop {
        buf.clear();
        let n = input.read_until(b'\n', &mut buf)?;
        if n == 0 {
            break;
        }
        let line = String::from_utf8_lossy(&buf);
        wr.write_paragraph(&line, &print_fn);
    }
    wr.flush_paragraph(&print_fn);

    Ok(())
}
