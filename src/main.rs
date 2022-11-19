use project::ff7;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ff7::decompress_lgp("./char.lgp")?;
    Ok(())
}
