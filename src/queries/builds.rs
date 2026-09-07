//! A module that formats the details of one (or multiple) build(s),
//! from urls such as <https://hydra.nixos.org/build/290062156>.
//!
//! This module is adapted from the `evals` module as the two are similar
//! in structure. The module is currently only used by the `builds` module,
//! hence the relevant interfaces are marked as `pub(super)`.

use serde::Serialize;

use crate::{EvalInput, FetchHydraReport, SoupFind, StatusIcon};

#[non_exhaustive]
#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone)]
pub(super) struct BuildReport {
    url: String,
    pub(super) inputs: Vec<EvalInput>,
    pub(super) log_url: Option<String>,
}

impl FetchHydraReport for BuildReport {
    fn get_url(&self) -> &str {
        &self.url
    }

    fn finish_with_error(self, status: String) -> Self {
        Self {
            inputs: vec![EvalInput {
                name: Some(StatusIcon::Warning.to_string()),
                value: Some(status),
                ..Default::default()
            }],
            ..self
        }
    }
}

impl BuildReport {
    pub(super) fn from_url(url: &str) -> Self {
        Self {
            url: url.to_string(),
            inputs: vec![],
            log_url: None,
        }
    }

    pub(super) fn fetch_and_read(self) -> anyhow::Result<Self> {
        let doc = self.fetch_document()?;
        let log_url = self
            .find_tbody(&doc, "div#tabs-summary table.info-table")
            .ok()
            .and_then(|tbody| {
                for row in tbody.find_all("tr") {
                    let Ok(heading) = row.find("th") else {
                        continue;
                    };
                    let heading: String = heading.text().collect();
                    if !heading.trim().contains("Logfile:") {
                        // we only care about the log file link for the moment
                        // we may add more information in the future
                        continue;
                    }
                    for link in row.find_all("a") {
                        let label: String = link.text().collect();
                        if label.trim() != "raw" {
                            continue;
                        }
                        let Some(href) = link.attr("href") else {
                            continue;
                        };
                        let url = reqwest::Url::parse(&self.url).ok()?.join(href).ok()?;
                        return Some(url.to_string());
                    }
                }
                None
            });
        let report = Self { log_url, ..self };
        let tbody = match report.find_tbody(&doc, "div#tabs-buildinputs") {
            // inputs are essential information, so exit early if this fails:
            Err(stat) => return Ok(stat),
            Ok(tbody) => tbody,
        };
        let inputs = EvalInput::from_tbody(tbody, &report.url)?;
        Ok(Self { inputs, ..report })
    }
}
