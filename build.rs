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

    #[allow(clippy::uninlined_format_args)]
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
    println!("cargo::rerun-if-changed=Cargo.lock");

    println!("{WARN}=auditing dependencies in Cargo.lock");
    let Ok(lock_file) = cargo_lock::Lockfile::load("./Cargo.lock") else {
        println!("{WARN}=failed to load Cargo.lock");
        return;
    };

    let mut patches = std::collections::HashMap::new();
    for pkg in lock_file
        .packages
        .into_iter()
        .filter(|pkg| pkg.source.is_some())
    {
        use cargo_lock::package::{GitReference, SourceKind};

        let src = pkg.source.unwrap();
        match src.kind() {
            SourceKind::Registry if src.is_default_registry() => {
                continue; // skip, do not warn
            }
            SourceKind::Git(git_reference) => match pkg.name.as_str() {
                "trycmd" | "snapbox" | "snapbox-macros" => {
                    let ref_string = match git_reference {
                        GitReference::Tag(s) | GitReference::Branch(s) | GitReference::Rev(s) => s,
                        #[allow(unreachable_patterns)]
                        _ => "",
                    };
                    let compare_url = format!(
                        "{}/compare/{}",
                        src.url().as_str().trim_end_matches(".git"),
                        ref_string
                    );
                    println!("{WARN}=* {}@{}: {}", pkg.name, pkg.version, compare_url);
                    let patch_url = format!("{compare_url}.patch");
                    patches
                        .entry(patch_url)
                        .or_insert(format!("{}@{}", pkg.name, pkg.version));
                    continue;
                }
                _ => {}
            },
            _ => {}
        };
        println!(
            "{WARN}=? unknown source: {}@{}: {src}",
            pkg.name, pkg.version
        );
    }

    for (patch, pkg) in patches {
        println!("{WARN}=saving diffs for the forked dependencies");
        download_patch(&patch, &pkg).unwrap_or_else(|e| {
            println!("{WARN}=failed to fetch patch: {e}");
        });
    }
}

fn download_patch(url: &str, pkg: &str) -> anyhow::Result<()> {
    use std::fs;

    let dir = "./patches";
    let path = format!("{dir}/{pkg}.patch");
    println!("{WARN}=* {path}: {url}");

    let contents = reqwest::blocking::get(url)?.error_for_status()?.text()?;
    fs::create_dir(dir).unwrap_or_default();
    fs::write(&path, contents)?;
    Ok(())
}
