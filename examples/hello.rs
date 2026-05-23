use figdriver::{FIGfont, Smusher};

fn main() {
    match run() {
        Ok(msg) => msg.iter().for_each(|x| println!("{}", x)),
        Err(e)  => println!("Error: {}", e),
    }
}

// This example loads the standard FIGlet font and writes a big hello world
// message like this:
//  _   _      _ _                            _     _ 
// | | | | ___| | | ___   __      _____  _ __| | __| |
// | |_| |/ _ \ | |/ _ \  \ \ /\ / / _ \| '__| |/ _` |
// |  _  |  __/ | | (_) |  \ V  V / (_) | |  | | (_| |
// |_| |_|\___|_|_|\___/    \_/\_/ \___/|_|  |_|\__,_|
                                                    
fn run() -> Result<Vec<String>, figdriver::Error> {
    let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/fonts/standard.flf";
    let font = FIGfont::from_path(&path)?;
    let mut sm = Smusher::new(&font);
    sm.push_str("Hello world");
    Ok(sm.get())
}
