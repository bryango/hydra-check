//! This build script extends the package version, i.e. `CARGO_PKG_VERSION`
//! with a commit hash suffix provided by the `version` environment variable,
//! which is currently set by nix in `package.nix`. This is useful for
//! identifying development builds.
//!
//! The script should never fail even if the `version` environment variable is
//! unset, in which case it should simply fall back to the default version
//! defined in `Cargo.toml`.

const WARN: &str = "cargo::warning";

fn main() {
    let version_from_nix = option_env!("version").unwrap_or_else(|| {
        println!("{WARN}=environment variable `version` not found");
        "" // empty &str
    });
    let version_from_cargo = env!("CARGO_PKG_VERSION");

    if version_from_nix.starts_with(version_from_cargo) {
        // use the hashed version string from nix
        println!("cargo::rustc-env=CARGO_PKG_VERSION={version_from_nix}");
    } else {
        println!(
            "{WARN}=inconsistent versioning: Cargo.toml={} vs package.nix={}",
            version_from_cargo, version_from_nix
        );
    };

    println!("cargo::rerun-if-env-changed=version");
}
