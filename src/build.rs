use std::{fmt::Display, time::Duration};

use anyhow::{anyhow, bail};
use colored::{ColoredString, Colorize};
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
struct PackageStatus<'a> {
    package: &'a str,
    args: Option<&'a ResolvedArgs>,
    url: String,
    builds: Vec<BuildStatus>,
}

impl<'a> PackageStatus<'a> {
    fn from_package_with_args(package: &'a str, args: &'a ResolvedArgs) -> Self {
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
            args: Some(args),
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
            let status = status.find("img")?.try_attr("title")?.into();
            let build_id = build.find("a")?.text().collect();
            let build_url = build.find("a")?.attr("href");
            let timestamp = timestamp.find("time")?.attr("datetime");
            let name = name.text().collect();
            let arch = arch.find("tt")?.text().collect();
            let success = status == "Succeeded";
            let icon = match success {
                true => StatusIcon::Success,
                false => StatusIcon::Failure,
            };
            let evals = true;
            builds.push(BuildStatus {
                icon,
                success,
                status,
                timestamp: timestamp.map(str::to_string),
                build_id: Some(build_id),
                build_url: build_url.map(str::to_string),
                name: Some(name),
                arch: Some(arch),
                evals,
            });
        }
        Ok(Self { builds, ..self })
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

impl From<&StatusIcon> for ColoredString {
    fn from(icon: &StatusIcon) -> Self {
        match icon {
            StatusIcon::Success => "✔".green(),
            StatusIcon::Failure => "✖".red(),
            StatusIcon::Queued => "⧖".yellow(),
            StatusIcon::Warning => "⚠".yellow(),
        }
    }
}

impl Display for StatusIcon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let icon = ColoredString::from(self).normal();
        write!(f, "{icon}")
    }
}

#[test]
fn serialize_success_icon() {
    let success_icon = serde_json::to_string(&StatusIcon::Success).unwrap();
    debug_assert_eq!(success_icon, r#""✔""#)
}

#[test]
fn fetch_and_parse() -> anyhow::Result<()> {
    use crate::Args;
    use clap::Parser;
    let args = Args::parse_from(["hydra-check", "--channel", "staging-next", "coreutils"])
        .guess_all_args()
        .unwrap();
    for package in args.packages.iter() {
        let pkg_stat = PackageStatus::from_package_with_args(package, &args);
        let pkg_stat = pkg_stat.fetch_and_parse()?;
        eprintln!("> build stats in json:");
        println!("{}", serde_json::to_string(&pkg_stat.builds)?);
    }
    Ok(())
}
