use hydra_check::HydraCheckCli;

fn main() -> anyhow::Result<()> {
    let success = HydraCheckCli::execute()?;
    if !success {
        std::process::exit(1);
    }
    Ok(())
}

/// Checks that console examples in README run successfully.
/// Updates the outputs automatically if the environment variable
/// `TRYCMD=overwrite`.
#[test]
#[ignore = "require internet connection, and not reproducible"]
#[cfg(feature = "trycmd")]
fn trycmd() {
    let test_status = std::env::var("TRYCMD").is_err_and(|_| unsafe {
        std::env::set_var("TRYCMD", "status"); // check exit status only
        true
    });
    trycmd::TestCases::new().case("README.md").run(); // run explicitly now
    test_status.then(|| unsafe {
        std::env::remove_var("TRYCMD");
    });
}
