use flexi_logger::Logger;
use hydra_check::{log_format, Args};

fn main() -> anyhow::Result<()> {
    Logger::try_with_str("info")?.format(log_format).start()?;
    let args = Args::parse_and_guess();
    println!("{:?}", args);
    Ok(())
}
