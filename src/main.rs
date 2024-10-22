use hydra_check::Args;

fn main() -> anyhow::Result<()> {
    let args = Args::parse_and_guess()?;
    println!("{:?}", args);
    Ok(())
}
