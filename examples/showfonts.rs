use std::error::Error;
use std::fs;
use std::path::PathBuf;
use figdriver::{FIGfont, Smusher};

fn main() {
    match run() {
        Ok(_)  => {},
        Err(e) => println!("Error: {}", e),
    }
}

// This example lists all FIGfonts in the fonts directory, like showfigfonts(6) utility
// distributed with FIGlet.

fn run() -> Result<(), Box<dyn Error>> {
    let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/fonts";
    let mut fonts: Vec<_> = fs::read_dir(&path)?
        .map(|x| x.unwrap().path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "flf"))
        .collect();
    fonts.sort();
    for f in fonts {
        show_font(f, &path)?;
    }
    Ok(())
}

fn show_font(p: PathBuf, prefix: &str) -> Result<(), Box<dyn Error>> {
    let font = FIGfont::from_path(p.to_str().unwrap())?;
    let name = p.strip_prefix(prefix)?.file_stem().unwrap().to_str().unwrap();
    println!("{}:", name); 
    let mut sm = Smusher::new(font);
    sm.push_str(name);
    for x in sm.get() {
        println!("{}", x);
    }
    println!("\n");
    Ok(())
}
