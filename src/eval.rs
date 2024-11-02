use std::fmt::Display;

use anyhow::{anyhow, bail};
use colored::Colorize;
use indexmap::IndexMap;
use log::{info, warn};
use regex::Regex;
use scraper::Html;
use serde::Serialize;
use serde_json::Value;
use serde_with::skip_serializing_none;

use crate::{
    args::Evaluation, BuildStatus, FetchHydra, ResolvedArgs, SoupFind, StatusIcon, TryAttr,
};

#[skip_serializing_none]
#[derive(Serialize, Clone, Default, Debug)]
struct EvalInput {
    name: Option<String>,
    #[serde(rename = "type")]
    input_type: Option<String>,
    value: Option<String>,
    revision: Option<String>,
    store_path: Option<String>,
}

impl Display for EvalInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(&self).expect("EvalInput should be serialized into json");
        let json: Value = serde_json::from_str(&json).unwrap();
        let strings: Vec<_> = ["name", "type", "value", "revision", "store_path"]
            .iter()
            .filter_map(|key| match &json[key] {
                Value::Null => None,
                // unquote the string:
                Value::String(value) => {
                    let key = match *key {
                        "name" => "input",
                        k => k,
                    };
                    Some(format!("{}: {}", key.bold(), value))
                }
                value => Some(format!("{}: {}", key.bold(), value)),
            })
            .collect();
        write!(f, "{}", strings.join("\n"))
    }
}

#[skip_serializing_none]
#[derive(Serialize, Clone)]
struct EvalInputChanges {
    input: String,
    description: String,
    url: Option<String>,
    revs: Option<(String, String)>,
    short_revs: Option<(String, String)>,
}

impl Display for EvalInputChanges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(&self).expect("EvalInput should be serialized into json");
        let json: Value = serde_json::from_str(&json).unwrap();
        let strings: Vec<_> = ["input", "description", "url", "revs"]
            .iter()
            .filter_map(|key| match &json[key] {
                Value::Null => None,
                // unquote the string:
                Value::String(value) => {
                    let key = match *key {
                        "input" => "changed_input",
                        "description" => "changes",
                        k => k,
                    };
                    Some(format!("{}: {}", key.bold(), value))
                }
                Value::Array(vec) => {
                    let texts: Vec<_> = vec.iter().filter_map(|x| x.as_str()).collect();
                    Some(format!("{}: {}", key.bold(), texts.join(" -> ")))
                }
                value => Some(format!("{}: {}", key.bold(), value)),
            })
            .collect();
        write!(f, "{}", strings.join("\n"))
    }
}

#[derive(Serialize, Clone)]
struct EvalDetails<'a> {
    #[serde(flatten)]
    eval: &'a Evaluation,
    url: String,
    inputs: Vec<EvalInput>,
    changes: Vec<EvalInputChanges>,
    aborted: Vec<BuildStatus>,
    now_fail: Vec<BuildStatus>,
    now_succeed: Vec<BuildStatus>,
    new: Vec<BuildStatus>,
    removed: Vec<BuildStatus>,
    still_fail: Vec<BuildStatus>,
    still_succeed: Vec<BuildStatus>,
    unfinished: Vec<BuildStatus>,
}

impl FetchHydra for EvalDetails<'_> {
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

impl<'a> From<&'a Evaluation> for EvalDetails<'a> {
    fn from(eval: &'a Evaluation) -> Self {
        let url = format!("https://hydra.nixos.org/eval/{}", eval.id);
        let url = match &eval.filter {
            Some(x) => format!("{url}?filter={x}"),
            None => url,
        };
        Self {
            eval,
            url,
            inputs: vec![],
            changes: vec![],
            aborted: vec![],
            now_fail: vec![],
            now_succeed: vec![],
            new: vec![],
            removed: vec![],
            still_fail: vec![],
            still_succeed: vec![],
            unfinished: vec![],
        }
    }
}

impl<'a> EvalDetails<'a> {
    fn parse_input_changes(doc: &Html) -> anyhow::Result<Vec<EvalInputChanges>> {
        let tables = doc.find_all("div#tabs-inputs table");
        let err = || {
            anyhow!(
                "could not parse the table of changed inputs in {:?}",
                tables.iter().map(|x| x.html()).collect::<Vec<_>>()
            )
        };
        // table of input changes:
        let table = tables.get(1).ok_or_else(err)?;
        let thead: Vec<String> = table
            .find("tr")?
            .find_all("th")
            .iter()
            .map(|x| x.text().collect())
            .collect();
        if !thead
            .iter()
            .all(|x| x.trim().contains("Input") || x.trim().contains("Changes"))
        {
            bail!(err());
        }
        let tbody = table.find_all("tr");
        let rows = tbody.get(1..).ok_or_else(err)?;
        let mut input_changes = Vec::new();
        for row in rows {
            let columns = row.find_all("td");
            let mut columns = columns.iter();
            let input: String = columns.next().ok_or_else(err)?.text().collect();
            let input = input.trim().to_string();

            let changes = columns.next().ok_or_else(err)?;
            let description: String = changes.text().collect();
            let description = description.trim().to_string();

            // the following entries are non-essential,
            // so we avoid using `?` for premature exits
            let url = changes
                .find("a")
                .ok()
                .and_then(|x| x.attr("href"))
                .map(|x| x.to_string());

            let revs = if let Some(url) = &url {
                // note that the returned url is not deterministic:
                // the position of the query parameters may float around
                let [rev1, rev2] = ["rev1", "rev2"].map(|rev_spec| {
                    let re = format!("^.*{rev_spec}=([0-9a-z]+).*$");
                    match Regex::new(&re).unwrap().captures(url).map(|x| x.extract()) {
                        Some((_, [rev])) if !rev.is_empty() => Some(rev.to_string()),
                        _ => None,
                    }
                });

                match (rev1, rev2) {
                    (Some(rev1), Some(rev2)) => Some((rev1, rev2)),
                    _ => None,
                }
            } else {
                None
            };

            let short_revs = if !description.is_empty() {
                match Regex::new("^([0-9a-z]+) to ([0-9a-z]+)$")
                    .unwrap()
                    .captures(&description)
                    .map(|x| x.extract())
                {
                    Some((_, [rev1, rev2])) if (!rev1.is_empty()) && (!rev2.is_empty()) => {
                        Some((rev1.to_string(), rev2.to_string()))
                    }
                    _ => None,
                }
            } else {
                None
            };

            input_changes.push(EvalInputChanges {
                input,
                description,
                url,
                revs,
                short_revs,
            });
        }
        Ok(input_changes)
    }

    fn parse_build_stats(&self, doc: &Html, selector: &str) -> anyhow::Result<Vec<BuildStatus>> {
        let err = || {
            anyhow!(
                "could not parse the table of build stats '{:?}' in {}",
                selector,
                doc.html()
            )
        };
        let tbody = match self.find_tbody(&doc, selector) {
            Err(stat) => bail!("{:?}", stat.inputs.first().ok_or_else(err)?.value),
            Ok(tbody) => tbody,
        };
        let mut builds: Vec<BuildStatus> = Vec::new();
        for row in tbody.find_all("tr") {
            let columns = row.find_all("td");
            let [status, build, job_name, timestamp, name, arch] = columns.as_slice() else {
                if Self::is_skipable_row(row)? {
                    continue;
                } else {
                    bail!("error while parsing Hydra status for {}", row.html());
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
            let job_name: String = job_name.text().collect();
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
                job_name: Some(job_name.trim().into()),
            });
        }
        Ok(builds)
    }

    fn fetch_and_read(self) -> anyhow::Result<Self> {
        let doc = self.fetch_document()?;
        let tbody = match self.find_tbody(&doc, "div#tabs-inputs") {
            // inputs are essential information, so exit early if this fails:
            Err(stat) => return Ok(stat),
            Ok(tbody) => tbody,
        };
        let mut inputs: Vec<EvalInput> = Vec::new();
        for row in tbody.find_all("tr") {
            let columns = row.find_all("td");
            let columns: Vec<_> = columns
                .iter()
                .map(|x| {
                    let text: String = x.text().collect();
                    match text.trim() {
                        x if x.is_empty() => None,
                        x => Some(x.to_string()),
                    }
                })
                .collect();
            let [name, input_type, value, revision, store_path] = columns.as_slice() else {
                if let Ok(true) = Self::is_skipable_row(row) {
                    info!(
                        "{}; for more information, please visit: {}",
                        "it appears that the result is truncated",
                        self.get_url()
                    );
                    continue;
                } else {
                    bail!(
                        "error while parsing inputs for eval {}: {:?}",
                        self.eval.id,
                        row.html()
                    );
                }
            };
            inputs.push(EvalInput {
                name: name.to_owned(),
                input_type: input_type.to_owned(),
                value: value.to_owned(),
                revision: revision.to_owned(),
                store_path: store_path.to_owned(),
            });
        }

        let changes = Self::parse_input_changes(&doc).unwrap_or_else(|err| {
            warn!("{}\n{}", err, err.backtrace());
            vec![]
        });

        let still_succeed = self
            .parse_build_stats(&doc, "div#tabs-still-succeed")
            .unwrap_or_else(|err| {
                warn!("{}\n{}", err, err.backtrace());
                vec![]
            });

        Ok(Self {
            inputs,
            changes,
            still_succeed,
            ..self
        })
    }
}

impl ResolvedArgs {
    pub(crate) fn fetch_and_print_evaluations(
        &self,
        evals: &Vec<Evaluation>,
    ) -> anyhow::Result<bool> {
        let mut indexmap = IndexMap::new();
        let evals = match &evals.is_empty() {
            false => evals.clone(),
            true => {
                info!(
                    "querying the latest evaluation of --jobset '{}'",
                    self.jobset
                );
                let err = || {
                    anyhow!(
                        "could not find the latest evaluation for --jobset '{}'",
                        self.jobset
                    )
                };
                eprintln!("");
                let id = self
                    .fetch_and_print_jobset(true)?
                    .ok_or_else(err)?
                    .to_string();
                eprintln!("");
                vec![Evaluation::guess_from_spec(&id)]
            }
        };
        for (idx, eval) in evals.iter().enumerate() {
            let stat = EvalDetails::from(eval);
            if self.url {
                println!("{}", stat.get_url());
                continue;
            }
            if !self.json {
                // print title first, then fetch
                if idx > 0 && !self.short {
                    println!(""); // vertical whitespace
                }
                println!(
                    "Evaluation {}{} {}",
                    stat.eval.id.to_string().bold(),
                    match &stat.eval.filter {
                        Some(x) => format!(" filtered by '{}'", x.bold()),
                        None => "".into(),
                    },
                    format!("@ {}", stat.get_url()).dimmed(),
                );
            }
            let stat = stat.fetch_and_read()?;
            if self.json {
                indexmap.insert(&stat.eval.spec, stat);
                continue;
            }
            for entry in &stat.inputs {
                println!(""); // vertical separation
                println!("{entry}");
            }
            for entry in &stat.changes {
                println!(""); // vertical separation
                println!("{entry}");
            }
            if !stat.still_succeed.is_empty() {
                println!("");
                println!("{}", stat.format_table(false, &stat.still_succeed));
            }
        }
        if self.json {
            println!("{}", serde_json::to_string_pretty(&indexmap)?);
        }
        Ok(true)
    }
}
