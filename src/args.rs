use clap::{arg, command, Parser};
use log::{error, info, warn};
use regex::Regex;
use std::{
    env::consts::{ARCH, OS},
    path::Path,
};

use crate::{constants, NixpkgsChannelVersion};

#[derive(Parser, Debug)]
#[command(author, version, verbatim_doc_comment)]
///
/// Check hydra.nixos.org for build status of packages
///
/// Other channels can be:
///   - unstable      - alias for nixos/trunk-combined (Default for Linux architectures)
///   - master        - alias for nixpkgs/trunk (Default for Darwin architectures)
///   - staging-next  - alias for nixpkgs/staging-next
///   - 24.05         - alias for nixos/release-24.05
///
/// Usually using the above as --channel arguments, should fit most usages.
/// However, you can use a verbatim jobset name such as:
///
///     nixpkgs/nixpkgs-24.05-darwin
///
/// Jobset names can be constructed with the project name (e.g. `nixos/` or `nixpkgs/`)
/// followed by a branch name. The available jobsets can be found at:
///   - https://hydra.nixos.org/project/nixos
///   - https://hydra.nixos.org/project/nixpkgs
///
pub struct Args {
    #[arg(conflicts_with = "eval")]
    packages: Vec<String>,

    /// Only print the hydra build url, then exit
    #[arg(long)]
    url: bool,

    /// Output json
    #[arg(long)]
    json: bool,

    /// Write only the latest build even if last build failed
    #[arg(short, long)]
    short: bool,

    /// System architecture to check
    #[arg(short, long)]
    arch: Option<String>,

    /// Channel to check packages for
    #[arg(short, long, default_value = "unstable")]
    channel: String,

    /// Specify jobset to check packages for
    #[arg(long, conflicts_with = "channel")]
    jobset: Option<String>,

    /// Print information about a specific evaluation
    #[arg(short, long)]
    eval: Option<String>,
}

impl Args {
    fn guess_arch(self) -> Self {
        let warn_if_unknown = |arch: &str| {
            if !Vec::from(constants::KNOWN_ARCHITECTURES).contains(&arch) {
                warn!(
                    "unknown '--arch {arch}', {}: {:?}",
                    "consider specify one of the following known architectures",
                    constants::KNOWN_ARCHITECTURES
                );
            }
        };
        if let Some(arch) = self.arch.clone() {
            warn_if_unknown(&arch);
            return self;
        }
        let arch = format!("{}-{}", ARCH, OS);
        info!("assuming '--arch {arch}'");
        warn_if_unknown(&arch);
        Self {
            arch: Some(arch),
            ..self
        }
    }

    fn guess_jobset(self) -> Self {
        if self.jobset.is_some() {
            return self;
        }
        // https://wiki.nixos.org/wiki/Channel_branches
        // https://github.com/NixOS/infra/blob/master/channels.nix
        let (trunk, combined) = ("nixpkgs/trunk", "nixos/trunk-combined");
        let jobset: String = match self.channel.as_str() {
            "master" | "nixpkgs-unstable" => trunk.into(),
            "nixos-unstable" => combined.into(),
            "nixos-unstable-small" => "nixos/unstable-small".into(),
            "unstable" => match Path::new("/etc/NIXOS").exists() {
                true => combined.into(), // NixOS
                false => trunk.into(),   // others
            },
            "stable" => {
                let ver = match NixpkgsChannelVersion::stable() {
                    Ok(version) => version,
                    Err(err) => {
                        error!(
                            "{}, {}.\n\n{}",
                            "could not fetch the stable release version number",
                            "please specify '--channel' or '--jobset' explicitly",
                            err
                        );
                        std::process::exit(1);
                    }
                };
                match self.arch.clone() {
                    // darwin
                    Some(x) if x.ends_with("darwin") => format!("nixpkgs/nixpkgs-{ver}-darwin"),
                    // others
                    _ => format!("nixos/release-{ver}"),
                }
            }
            x if x.starts_with("staging-next") => format!("nixpkgs/{x}"),
            x if Regex::new(r"^[0-9]+\.[0-9]+$").unwrap().is_match(x) => {
                format!("nixos/release-{x}")
            }
            x if Regex::new(r"^nixos-[0-9]+\.[0-9]+").unwrap().is_match(x) => {
                x.replacen("nixos", "nixos/release", 1)
            }
            x if Regex::new(r"^nixpkgs-[0-9]+\.[0-9]+").unwrap().is_match(x) => {
                x.replacen("nixpkgs", "nixpkgs/nixpkgs", 1)
            }
            _ => self.channel.clone(),
        };
        info!("'--channel {}' implies '--jobset {}'", self.channel, jobset);
        Self {
            jobset: Some(jobset),
            ..self
        }
    }

    pub fn parse_and_guess() -> Self {
        let args = Self::parse();
        let args = args.guess_arch();
        let args = args.guess_jobset();
        args
    }
}

#[test]
fn guess_jobset() {
    let aliases = [
        ("24.05", "nixos/release-24.05"),
        ("nixos-23.05", "nixos/release-23.05"),
        ("nixos-23.11-small", "nixos/release-23.11-small"),
    ];
    for (channel, jobset) in aliases {
        eprintln!("{channel} => {jobset}");
        let args = Args::parse_from(["hydra-check", "--channel", channel]).guess_jobset();
        debug_assert_eq!(args.jobset, Some(jobset.into()))
    }
}

#[test]
#[ignore = "require internet connection"]
fn guess_stable() {
    let args = Args::parse_from(["hydra-check", "--channel", "stable"]).guess_jobset();
    eprintln!("{:?}", args.jobset)
}
