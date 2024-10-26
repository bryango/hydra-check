use std::{fmt::Display, time::Duration};

use anyhow::bail;
use colored::{ColoredString, Colorize};
use comfy_table::Table;
use scraper::Html;
use serde::Serialize;
use serde_with::{skip_serializing_none, SerializeDisplay};

use crate::{ResolvedArgs, SoupFind, TryAttr};

#[skip_serializing_none]
#[derive(Serialize, Debug, Default)]
/// Status of a single build attempt, can be serialized a JSON entry
pub struct BuildStatus {
    icon: StatusIcon,
    pub success: bool,
    status: String,
    timestamp: Option<String>,
    build_id: Option<String>,
    build_url: Option<String>,
    name: Option<String>,
    arch: Option<String>,
    evals: bool,
}

/// Container for the build status and metadata of a package
pub struct PackageStatus<'a> {
    package: &'a str,
    args: &'a ResolvedArgs,
    url: String,
    /// Status of recent builds of the package
    pub builds: Vec<BuildStatus>,
}

impl<'a> PackageStatus<'a> {
    /// Initializes the status container with the resolved package name
    /// and the resolved command line arguments.
    pub fn from_package_with_args(package: &'a str, args: &'a ResolvedArgs) -> Self {
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
            builds: vec![],
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
                // sanitize the text a little bit
                let status: Vec<&str> = status.lines().map(str::trim).collect();
                let status: String = status.join(" ");
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
                    bail!(
                        "error while parsing Hydra status for package {}: {:?}",
                        self.package,
                        row
                    );
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
            let build_id = build.find("a")?.text().collect();
            let build_url = build.find("a")?.attr("href");
            let timestamp = timestamp.find("time")?.attr("datetime");
            let name = name.text().collect();
            let arch = arch.find("tt")?.text().collect();
            let success = status == "Succeeded";
            let icon = match (success, status) {
                (true, _) => StatusIcon::Success,
                (false, "Cancelled") => StatusIcon::Cancelled,
                (false, _) => StatusIcon::Failure,
            };
            let evals = true;
            builds.push(BuildStatus {
                icon,
                success,
                status: status.into(),
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

    /// Fetches the package build status from hydra.nixos.org and formats
    /// the result according to the command line specs.
    pub fn fetch_and_format(self) -> anyhow::Result<String> {
        if self.args.url {
            return Ok(self.get_url().into());
        }
        let stat = self.fetch_and_parse()?;
        if stat.args.json {
            let json_string = serde_json::to_string_pretty(&stat.builds)?;
            return Ok(json_string);
        }
        let title = format!(
            "Build Status for {} on jobset {}\n{}\n",
            stat.package.bold(),
            stat.args.jobset.bold(),
            stat.get_url().dimmed()
        );

        let mut table = Table::new();
        table.load_preset(comfy_table::presets::NOTHING);
        for build in stat.builds {
            table.add_row(build.as_vec());
            if stat.args.short {
                break;
            }
        }
        for column in table.column_iter_mut() {
            column.set_padding((0, 1));
            // column.set_constraint(comfy_table::ColumnConstraint::ContentWidth);
            break; // only for the first column
        }
        Ok(title + table.to_string().as_str())
    }
}

impl BuildStatus {
    fn as_vec(&self) -> Vec<ColoredString> {
        let mut row = Vec::new();
        let icon = ColoredString::from(&self.icon);
        let status = match (self.evals, self.success) {
            (false, _) => format!("{icon} {}", self.status),
            (true, false) => format!("{icon} ({})", self.status),
            (true, true) => format!("{icon}"),
        };
        row.push(status.into());
        let details = if self.evals {
            let name = self.name.clone().unwrap_or_default().into();
            let timestamp = self
                .timestamp
                .clone()
                .unwrap_or_default()
                .split_once('T')
                .unwrap_or_default()
                .0
                .into();
            let build_url = self.build_url.clone().unwrap_or_default().dimmed();
            &[name, timestamp, build_url]
        } else {
            &Default::default()
        };
        row.extend_from_slice(details);
        row
    }
}

#[derive(SerializeDisplay, Debug, Clone, Default)]
enum StatusIcon {
    Success,
    Failure,
    Cancelled,
    Queued,
    #[default]
    Warning,
}

impl From<&StatusIcon> for ColoredString {
    fn from(icon: &StatusIcon) -> Self {
        match icon {
            StatusIcon::Success => "✔".green(),
            StatusIcon::Failure => "✖".red(),
            StatusIcon::Cancelled => "⏹".red(),
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

// #[test]
// fn fetch_and_parse() -> anyhow::Result<()> {
//     use crate::Args;
//     use clap::Parser;
//     let args = Args::parse_from([
//         "hydra-check",
//         "--channel",
//         "staging-next",
//         "coreutils",
//         "rust-cbindgen",
//         // "--json",
//         // "--short",
//     ])
//     .guess_all_args()
//     .unwrap();
//     for (idx, package) in args.queries.iter().enumerate() {
//         let pkg_stat = PackageStatus::from_package_with_args(package, &args);
//         if idx > 0 {
//             println!("");
//         }
//         println!("{}", pkg_stat.fetch_and_format()?);
//     }
//     Ok(())
// }
