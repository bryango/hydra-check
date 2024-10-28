use std::collections::HashMap;

use anyhow::bail;
use colored::{ColoredString, Colorize};
use comfy_table::Table;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{FetchData, ResolvedArgs, SoupFind, StatusIcon, TryAttr};

#[skip_serializing_none]
#[derive(Serialize, Debug, Default, Clone)]
/// Status of a single build attempt, can be serialized to a JSON entry
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

impl BuildStatus {
    fn format_as_vec(&self) -> Vec<ColoredString> {
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

#[derive(Clone)]
/// Container for the build status and metadata of a package
pub struct PackageStatus<'a> {
    package: &'a str,
    url: String,
    /// Status of recent builds of the package
    pub builds: Vec<BuildStatus>,
}

impl FetchData for PackageStatus<'_> {
    fn get_url(&self) -> &str {
        &self.url
    }

    fn finish_with_error(self, status: String) -> Self {
        Self {
            builds: vec![BuildStatus {
                icon: StatusIcon::Warning,
                status,
                ..Default::default()
            }],
            ..self
        }
    }
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
            url,
            builds: vec![],
        }
    }

    fn fetch_and_parse(self) -> anyhow::Result<Self> {
        let doc = self.fetch_document()?;
        let tbody = match self.find_tbody(&doc) {
            Err(stat) => return Ok(stat),
            Ok(tbody) => tbody,
        };
        let mut builds: Vec<BuildStatus> = Vec::new();
        for row in tbody.find_all("tr") {
            let columns = row.find_all("td");
            let [status, build, timestamp, name, arch] = columns.as_slice() else {
                if Self::is_skipable_row(row)? {
                    continue;
                } else {
                    bail!(
                        "error while parsing Hydra status for package '{}': {:?}",
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
                (true, _) => StatusIcon::Succeeded,
                (false, "Cancelled") => StatusIcon::Cancelled,
                (false, _) => StatusIcon::Failed,
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
}

impl ResolvedArgs {
    /// Fetches packages build status from hydra.nixos.org and prints
    /// the result according to the command line specs.
    pub fn fetch_and_print_packages(&self, packages: &Vec<String>) -> anyhow::Result<bool> {
        let mut status = true;
        let mut hashmap = HashMap::new();
        for (idx, package) in packages.iter().enumerate() {
            let stat = PackageStatus::from_package_with_args(package, self);
            if self.url {
                println!("{}", stat.get_url());
                continue;
            }
            let stat = stat.fetch_and_parse()?;
            let first_stat = stat.builds.get(0);
            let success = first_stat.is_some_and(|build| build.success);
            if !success {
                status = false;
            }
            if self.json {
                match self.short {
                    true => hashmap.insert(
                        stat.package,
                        match first_stat {
                            Some(x) => vec![x.to_owned()],
                            None => vec![],
                        },
                    ),
                    false => hashmap.insert(stat.package, stat.builds),
                };
                continue; // print later
            }
            if idx > 0 && !self.short {
                println!("");
            }
            println!(
                "Build Status for {} on jobset {}{}",
                stat.package.bold(),
                self.jobset.bold(),
                match self.short && success {
                    true => "".into(),
                    false => format!("\n{}", stat.get_url().dimmed()),
                }
            );
            let mut table = Table::new();
            table.load_preset(comfy_table::presets::NOTHING);
            for build in stat.builds {
                table.add_row(build.format_as_vec());
                if self.short {
                    break;
                }
            }
            for column in table.column_iter_mut() {
                column.set_padding((0, 1));
                // column.set_constraint(comfy_table::ColumnConstraint::ContentWidth);
                break; // only for the first column
            }
            println!("{}", table.trim_fmt())
        }
        if self.json {
            println!("{}", serde_json::to_string_pretty(&hashmap)?);
        }
        Ok(status)
    }
}
