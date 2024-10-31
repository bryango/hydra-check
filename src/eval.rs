use std::fmt::Display;

use anyhow::{anyhow, bail};
use colored::Colorize;
use indexmap::IndexMap;
use log::info;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{args::Evaluation, BuildStatus, FetchHydra, ResolvedArgs, SoupFind, StatusIcon};

#[skip_serializing_none]
#[derive(Serialize, Clone, Default)]
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
        use serde_json::Value;

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
    url: String,
    revs: (String, String),
    short_revs: (String, String),
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
    type Status = BuildStatus;

    fn name(&self) -> &str {
        self.eval.spec.as_str()
    }

    fn get_url(&self) -> &str {
        &self.url
    }

    fn entries(&self) -> &Vec<Self::Status> {
        todo!()
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
    fn fetch_and_read(self) -> anyhow::Result<Self> {
        let doc = self.fetch_document()?;
        let tbody = match self.find_tbody(&doc, "div#tabs-inputs") {
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
                if Self::is_skipable_row(row)? {
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
                        row
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
        Ok(Self { inputs, ..self })
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
            for entry in stat.inputs {
                println!(""); // vertical separation
                println!("{entry}");
            }
        }
        if self.json {
            println!("{}", serde_json::to_string_pretty(&indexmap)?);
        }
        Ok(true)
    }
}
