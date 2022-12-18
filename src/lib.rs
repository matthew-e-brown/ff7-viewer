mod drawing;
mod math;
mod ff7;

use std::fs::File;

use ff7::ModelFile;
use ff7::decompress_lgp;


pub fn run(filepath: &str) -> Result<(), String> {
    let mut file = File::open(filepath).or(Err(format!("Could not open file {filepath}.")))?;
    let archive = match decompress_lgp(&mut file) {
        Err(e) => Err(e.to_string()),   // format error if error
        Ok(a) => Ok(a),                 // regular return otherwise
    }?;

    // List of all the HRC files, we will let the user pick one to view
    let hierarchies = {
        let mut list: Vec<_> = archive
            .iter()
            .filter_map(|(key, value)| match value {
                ModelFile::Hierarchy(_) => Some(key),
                _ => None,
            })
            .collect();
        list.sort();
        list
    };



    Ok(())
}
