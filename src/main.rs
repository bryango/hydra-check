use flexi_logger::Logger;
use hydra_check::{args::Args, log_format};

fn main() -> anyhow::Result<()> {
    Logger::try_with_str("info")?.format(log_format).start()?;
    let args = Args::parse_and_guess();
    println!("{:?}", args);
    Ok(())
}
