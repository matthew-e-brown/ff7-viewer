use std::{fs::File, io::Read};

use extract::{LGPFile, Parse};

fn main() {
    let data = {
        let mut buff = Vec::new();
        let mut file = File::open("char.lgp").expect("Couldn't open");
        file.read_to_end(&mut buff).expect("Couldn't read");
        buff
    };

    let file = LGPFile::from_bytes(&data);
    println!("{:#?}", file);
}
