use std::{fmt::Display, time::Duration};

use anyhow::{anyhow, bail};
use scraper::Html;
use serde::Serialize;
use serde_with::{skip_serializing_none, SerializeDisplay};

use crate::{ResolvedArgs, SoupFind, TryAttr};

#[skip_serializing_none]
#[derive(Serialize, Debug, Default)]
pub struct BuildStatus {
    icon: StatusIcon,
    success: bool,
    status: String,
    timestamp: Option<String>,
    build_id: Option<String>,
    build_url: Option<String>,
    name: Option<String>,
    arch: Option<String>,
    evals: bool,
}

#[derive(Default)]
struct PackageStatus {
    package: String,
    args: ResolvedArgs,
    url: String,
    builds: Vec<BuildStatus>,
}

impl PackageStatus {
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

    fn get_url(&self) -> &str {
        &self.url
    }

    fn fetch_data(&self) -> anyhow::Result<String> {
        let text = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()?
            .get(self.get_url())
            .send()?
            .error_for_status()?
            .text()?;
        Ok(text)
    }

    fn fetch_and_parse(self) -> anyhow::Result<Self> {
        let document = self.fetch_data()?;
        let doc = Html::parse_document(&document);
        let tbody = match doc.find("tbody") {
            Err(_) => {
                // either the package was not evaluated (due to being e.g. unfree)
                // or the package does not exist
                let status: String = if let Ok(alert) = doc.find("div.alert") {
                    alert.text().collect()
                } else {
                    format!("Unknown Hydra Error found at {}", self.get_url())
                };
                return Ok(Self {
                    builds: vec![BuildStatus {
                        icon: StatusIcon::Warning,
                        status,
                        ..Default::default()
                    }],
                    ..self
                });
            }
            Ok(tbody) => tbody,
        };
        let mut builds: Vec<BuildStatus> = Vec::new();
        for row in tbody.find_all("tr") {
            let err = || anyhow!("error parsing Hydra status: {:?}", row);
            let columns = row.find_all("td");
            let [status, build, timestamp, name, arch] = columns.as_slice() else {
                if row
                    .find("td")?
                    .find("a")?
                    .try_attr("href")?
                    .ends_with("/all")
                {
                    continue;
                } else {
                    bail!(err());
                }
            };
            if let Ok(span_status) = status.find("span") {
                let span_status: String = span_status.text().collect();
                let status = if span_status.trim() == "Queued" {
                    "Queued: no build has been attempted for this package yet (still queued)"
                        .to_string()
                } else {
                    format!("Unknown Hydra status: {span_status}")
                };
                builds.push(BuildStatus {
                    icon: StatusIcon::Queued,
                    status,
                    ..Default::default()
                });
                continue;
            }
            let status = status.find("img")?.try_attr("title")?;
            let build_id: String = build.find("a")?.text().collect();
            let build_url = build.find("a")?.try_attr("href")?;
            let timestamp = timestamp.find("time")?.try_attr("datetime")?;
            todo!()
        }
        todo!()
    }
}

#[derive(SerializeDisplay, Debug, Clone, Default)]
enum StatusIcon {
    Success,
    Failure,
    Queued,
    #[default]
    Warning,
}

impl Display for StatusIcon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let icon = match self {
            Self::Success => "✔",
            Self::Failure => "✖",
            Self::Queued => "⧖",
            Self::Warning => "⚠",
        };
        write!(f, "{icon}")
    }
}

#[test]
fn serialize_success_icon() {
    let success_icon = serde_json::to_string(&StatusIcon::Success).unwrap();
    debug_assert_eq!(success_icon, r#""✔""#)
}
