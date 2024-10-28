use hydra_check::Cli;

fn main() -> anyhow::Result<()> {
    let success = Cli::parse_and_guess()?.fetch_and_print()?;
    if !success {
        std::process::exit(1);
    }
    Ok(())
}
