use anyhow::bail;
use colored::{ColoredString, Colorize};
use indexmap::IndexMap;
use log::warn;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{FetchHydra, FormatVecColored, ResolvedArgs, SoupFind, StatusIcon, TryAttr};

#[skip_serializing_none]
#[derive(Serialize, Debug, Default, Clone)]
/// Status of a single build attempt, can be serialized to a JSON entry
pub(crate) struct BuildStatus {
    pub(crate) icon: StatusIcon,
    pub(crate) success: bool,
    pub(crate) status: String,
    pub(crate) timestamp: Option<String>,
    pub(crate) build_id: Option<String>,
    pub(crate) build_url: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) arch: Option<String>,
    pub(crate) evals: bool,
    pub(crate) job_name: Option<String>,
}

impl FormatVecColored for BuildStatus {
    fn format_as_vec(&self) -> Vec<ColoredString> {
        let mut row = Vec::new();
        let icon = ColoredString::from(&self.icon);
        let status = match (self.evals, self.success) {
            (false, _) => format!("{icon} {}", self.status),
            (true, false) => format!("{icon} ({})", self.status),
            (true, true) => format!("{icon}"),
        };
        row.push(status.into());
        match &self.job_name {
            Some(job_name) => row.push(job_name.as_str().into()),
            None => {}
        };
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
            &[name, timestamp]
        } else {
            &Default::default()
        };
        row.extend_from_slice(details);
        let build_url = self.build_url.clone().unwrap_or_default().dimmed();
        row.push(build_url);
        row
    }
}

#[derive(Clone)]
/// Container for the build status and metadata of a package
struct PackageStatus<'a> {
    package: &'a str,
    url: String,
    /// Status of recent builds of the package
    builds: Vec<BuildStatus>,
}

impl FetchHydra for PackageStatus<'_> {
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
            url,
            builds: vec![],
        }
    }

    fn fetch_and_read(self) -> anyhow::Result<Self> {
        let doc = self.fetch_document()?;
        let tbody = match self.find_tbody(&doc, "") {
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
                        "error while parsing Hydra status for package '{}': {}",
                        self.package,
                        row.html()
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
                job_name: None,
            });
        }
        Ok(Self { builds, ..self })
    }
}

impl ResolvedArgs {
    pub(crate) fn fetch_and_print_packages(&self, packages: &Vec<String>) -> anyhow::Result<bool> {
        let mut status = true;
        let mut indexmap = IndexMap::new();
        for (idx, package) in packages.iter().enumerate() {
            let stat = PackageStatus::from_package_with_args(package, self);
            if self.url {
                println!("{}", stat.get_url());
                continue;
            }
            let url_dimmed = stat.get_url().dimmed();
            if !self.json {
                // print title first, then fetch
                if idx > 0 && !self.short {
                    println!(""); // vertical whitespace
                }
                println!(
                    "Build Status for {} on jobset {}",
                    stat.package.bold(),
                    self.jobset.bold(),
                );
                if !self.short {
                    println!("{url_dimmed}");
                }
            }
            let stat = stat.fetch_and_read()?;
            let first_stat = stat.builds.first();
            let success = first_stat.is_some_and(|build| build.success);
            if !success {
                status = false;
            }
            if self.json {
                match self.short {
                    true => indexmap.insert(
                        stat.package,
                        match first_stat {
                            Some(x) => vec![x.to_owned()],
                            None => vec![],
                        },
                    ),
                    false => indexmap.insert(stat.package, stat.builds),
                };
                continue; // print later
            }
            println!("{}", stat.format_table(self.short, &stat.builds));
            if !success && self.short {
                warn!("latest build failed, check out: {}", url_dimmed)
            }
        }
        if self.json {
            println!("{}", serde_json::to_string_pretty(&indexmap)?);
        }
        Ok(status)
    }
}
