use hydra_check::{Args, PackageStatus, Queries, ResolvedArgs};
use std::borrow::Borrow;

fn query_packages(packages: &Vec<String>, args: &ResolvedArgs) -> anyhow::Result<bool> {
    let mut success = true;
    for (idx, package) in packages.iter().enumerate() {
        let pkg_stat = PackageStatus::from_package_with_args(package, &args);
        if idx > 0 {
            println!("");
        }
        if !pkg_stat.builds.get(0).is_some_and(|build| build.success) {
            success = false;
        }
        println!("{}", pkg_stat.fetch_and_format()?);
    }
    Ok(success)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse_and_guess()?;
    let success = match args.queries.borrow() {
        Queries::Jobset => todo!(),
        Queries::Packages(packages) => query_packages(packages, &args)?,
        Queries::Evals(evals) => todo!(),
    };
    if !success {
        std::process::exit(1);
    }
    Ok(())
}
