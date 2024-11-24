use hydra_check::HydraCheckCli;

fn main() -> anyhow::Result<()> {
    let success = HydraCheckCli::execute()?;
    if !success {
        std::process::exit(1);
    }
    Ok(())
}

/// Updates console examples in README automatically,
/// if the environment variable `TRYCMD=overwrite`.
#[test]
#[ignore = "require internet connection, and not reproducible"]
fn trycmd() {
    trycmd::TestCases::new().case("README.md");
}
