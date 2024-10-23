use std::{fmt::Display, time::Duration};

use serde::Serialize;
use serde_with::{skip_serializing_none, SerializeDisplay};

use crate::ResolvedArgs;

#[skip_serializing_none]
#[derive(Serialize, Debug, Default)]
pub struct BuildStatus {
    #[serde(skip)]
    package: String,
    #[serde(skip)]
    args: ResolvedArgs,
    icon: StatusIcon,
    success: bool,
    status: String,
    timestamp: String,
    build_id: String,
    build_url: String,
    name: String,
    arch: String,
    evals: bool,
    url: String,
}

impl BuildStatus {
    fn from_package_with_args(package: String, args: ResolvedArgs) -> Self {
        //
        // Examples:
        // - https://hydra.nixos.org/job/nixos/release-19.09/nixpkgs.hello.x86_64-linux/latest
        // - https://hydra.nixos.org/job/nixos/release-19.09/nixos.tests.installer.simpleUefiGrub.aarch64-linux
        // - https://hydra.nixos.org/job/nixpkgs/trunk/hello.x86_64-linux/all
        //
        // There is also {url}/all which is a lot slower.
        //
        let url = format!("https://hydra.nixos.org/job/{}/{}", args.jobset, package);
        Self {
            package,
            args,
            url,
            ..Default::default()
        }
    }

    fn fetch_data(self) -> anyhow::Result<String> {
        let text = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()?
            .get(self.url)
            .send()?
            .error_for_status()?
            .text()?;
        Ok(text)
    }
}

#[derive(SerializeDisplay, Debug, Clone, Default)]
enum StatusIcon {
    Success,
    Failure,
    #[default]
    Unknown,
}

impl Display for StatusIcon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let icon = match self {
            Self::Success => "✔",
            Self::Failure => "✖",
            Self::Unknown => "⚠",
        };
        write!(f, "{icon}")
    }
}

#[test]
fn serialize_success_icon() {
    let success_icon = serde_json::to_string(&StatusIcon::Success).unwrap();
    debug_assert_eq!(success_icon, r#""✔""#)
}
