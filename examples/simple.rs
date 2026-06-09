use std::io::{self, BufRead};
use figdriver::{Align, Control, FIGfont, Smusher, Wrapper};

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => eprintln!("Error: {}", e),
    }
}

// This example reads a text input from stdin and renders the text using the
// small FIGfont, wrapping lines if necessary.

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

    // Read input from stdin and render each line independently.
    // write_line handles clear, tokenization, and flush internally.
    let input = io::BufReader::new(io::stdin());
    for line in input.lines() {
        wr.write_line(&line?, &print_fn);
    }
    Ok(())
}
