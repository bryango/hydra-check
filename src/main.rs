use hydra_check::{Args, PackageStatus};

fn main() -> anyhow::Result<()> {
    let args = Args::parse_and_guess()?;
    let mut success = true;
    for (idx, package) in args.packages.iter().enumerate() {
        let pkg_stat = PackageStatus::from_package_with_args(package, &args);
        if idx > 0 {
            println!("");
        }
        if !pkg_stat.builds.get(0).is_some_and(|build| build.success) {
            success = false;
        }
        println!("{}", pkg_stat.fetch_and_print()?);
    }
    if !success {
        std::process::exit(1);
    }
    Ok(())
}
