use project::ff7;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Info).env().init()?;
    ff7::decompress_lgp("./char.lgp")?;
    Ok(())
}
