use hydra_check::{Args, JobsetStatus, Queries, ResolvedArgs};
use std::borrow::Borrow;

fn query_jobset(args: &ResolvedArgs) -> anyhow::Result<bool> {
    let jobset_stat = JobsetStatus::from(args);
    let output = jobset_stat.fetch_and_format()?;
    println!("{}", output);
    Ok(true)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse_and_guess()?;
    let success = match args.queries.borrow() {
        Queries::Jobset => query_jobset(&args)?,
        Queries::Packages(packages) => args.fetch_and_print_packages(packages)?,
        Queries::Evals(evals) => todo!(),
    };
    if !success {
        std::process::exit(1);
    }
    Ok(())
}
