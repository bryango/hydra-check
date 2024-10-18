use clap::{arg, command, Parser};
use log::info;
use std::env::consts::{ARCH, OS};

#[derive(Parser, Debug)]
#[command(author, version, verbatim_doc_comment)]
///
/// Check hydra.nixos.org for build status of packages
///
/// Other channels can be:
///   - unstable  - alias for nixos/trunk-combined (Default for Linux architectures)
///   - master    - alias for nixpkgs/trunk (Default for Darwin architectures)
///   - staging   - alias for nixos/staging
///   - 24.05     - alias for nixos/release-24.05
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
        if self.arch.is_some() {
            return self;
        }
        let arch = format!("{}-{}", ARCH, OS);
        info!("assuming '--arch {arch}'");
        Self {
            arch: Some(arch),
            ..self
        }
    }

    fn guess_jobset(self) -> Self {
        if self.jobset.is_some() {
            return self;
        }
        let jobset: String = match self.channel.as_str() {
            "unstable" => match self.arch.clone().is_some_and(|x| x.ends_with("darwin")) {
                true => "nixpkgs/trunk".into(),
                false => "nixos/trunk-combined".into(),
            },
            "master" => "nixpkgs/trunk".into(),
            x if x.starts_with("staging-next") => format!("nixpkgs/{x}"),
            _ => match self.channel.chars().next().is_some_and(|x| x.is_numeric()) {
                true => format!("nixos/release-{}", self.channel),
                false => self.channel.clone(),
            },
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
