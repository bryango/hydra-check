use hydra_check::Cli;

fn main() -> anyhow::Result<()> {
    let success = Cli::execute()?;
    if !success {
        std::process::exit(1);
    }
    Ok(())
}
