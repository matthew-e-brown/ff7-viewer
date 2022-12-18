use std::env;

use ff7_viewer::run;


pub fn main() -> Result<(), String> {
    let mut args = env::args();
    args.next(); // skip program name

    match args.next() {
        Some(path) => run(&path),
        None => Err("Missing required argument - path to LGP file.".to_owned()),
    }
}
