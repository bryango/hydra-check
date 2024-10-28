use hydra_check::{Args, Queries};
use std::borrow::Borrow;

fn main() -> anyhow::Result<()> {
    let args = Args::parse_and_guess()?;
    let success = match args.queries.borrow() {
        Queries::Jobset => args.fetch_and_print_jobset()?,
        Queries::Packages(packages) => args.fetch_and_print_packages(packages)?,
        Queries::Evals(evals) => todo!(),
    };
    if !success {
        std::process::exit(1);
    }
    Ok(())
}
