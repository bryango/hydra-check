use anyhow::bail;
use colored::{ColoredString, Colorize};
use comfy_table::Table;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{FetchData, ResolvedArgs, SoupFind, StatusIcon, TryAttr};

#[skip_serializing_none]
#[derive(Serialize, Debug, Default, Clone)]
/// Status of a single evaluation, can be serialized to a JSON entry
pub struct EvalStatus {
    icon: StatusIcon,
    finished: Option<bool>,
    id: Option<u64>,
    url: Option<String>,
    datetime: Option<String>,
    relative: Option<String>,
    timestamp: Option<u64>,
    status: String,
    short_rev: Option<String>,
    input_changes: Option<String>,
    succeeded: Option<u64>,
    failed: Option<u64>,
    queued: Option<u64>,
    delta: Option<String>,
}

impl EvalStatus {
    fn format_as_vec(&self) -> Vec<ColoredString> {
        let mut row = Vec::new();
        let icon = ColoredString::from(&self.icon);
        let description = match &self.input_changes {
            Some(x) => x,
            None => &self.status,
        };
        row.push(format!("{icon} {description}").into());
        let details = if self.url.is_some() {
            let relative = self.relative.clone().unwrap_or_default().into();
            let statistics = [
                (StatusIcon::Succeeded, self.succeeded),
                (StatusIcon::Failed, self.failed),
                (StatusIcon::Queued, self.queued),
            ];
            let [suceeded, failed, queued] = statistics.map(|(icon, text)| -> ColoredString {
                format!(
                    "{} {}",
                    ColoredString::from(&icon),
                    text.unwrap_or_default()
                )
                .into()
            });
            let queued = match self.queued.unwrap_or_default() {
                x if x != 0 => queued.bold(),
                _ => queued.normal(),
            };
            let delta = format!("Δ {}", self.delta.clone().unwrap_or_default());
            let delta = match self.delta.clone().unwrap_or("?".into()) {
                x if x.starts_with("+") => delta.green(),
                x if x.starts_with("-") => delta.red(),
                _ => delta.into(),
            };
            let url = self.url.clone().unwrap_or_default().dimmed();
            &[relative, suceeded, failed, queued, delta, url]
        } else {
            &Default::default()
        };
        row.extend_from_slice(details);
        row
    }
}

#[derive(Clone)]
/// Container for the eval status and metadata of a jobset
pub struct JobsetStatus<'a> {
    args: &'a ResolvedArgs,
    url: String,
    /// Status of recent evaluations of the jobset
    pub evals: Vec<EvalStatus>,
}

impl FetchData for JobsetStatus<'_> {
    fn get_url(&self) -> &str {
        &self.url
    }

    fn finish_with_error(self, status: String) -> Self {
        Self {
            evals: vec![EvalStatus {
                icon: StatusIcon::Warning,
                status,
                ..Default::default()
            }],
            ..self
        }
    }
}

impl<'a> From<&'a ResolvedArgs> for JobsetStatus<'a> {
    fn from(args: &'a ResolvedArgs) -> Self {
        let url = format!("https://hydra.nixos.org/jobset/{}/evals", args.jobset);
        Self {
            args,
            url,
            evals: vec![],
        }
    }
}

impl<'a> JobsetStatus<'a> {
    fn fetch_and_parse(self) -> anyhow::Result<Self> {
        let doc = self.fetch_document()?;
        let tbody = match self.find_tbody(&doc) {
            Err(stat) => return Ok(stat),
            Ok(tbody) => tbody,
        };
        let mut evals: Vec<EvalStatus> = Vec::new();
        for row in tbody.find_all("tr") {
            let columns = row.find_all("td");
            let [eval_id, timestamp, input_changes, succeeded, failed, queued, delta] =
                columns.as_slice()
            else {
                if Self::is_skipable_row(row)? {
                    continue;
                } else {
                    bail!(
                        "error while parsing Hydra status for jobset '{}': {:?}",
                        self.args.jobset,
                        row
                    );
                }
            };

            let url = eval_id.find("a")?.try_attr("href")?;
            let eval_id: String = eval_id.text().collect();
            let id: u64 = eval_id.parse()?;

            let time = timestamp.find("time")?;
            let date = time.try_attr("datetime")?;
            let relative = time.text().collect();
            let timestamp = time.try_attr("data-timestamp")?;
            let timestamp: u64 = timestamp.parse()?;

            let status: String = input_changes
                .find("span")
                .map(|x| x.text().collect())
                .unwrap_or_default();

            let short_rev = input_changes.find("tt")?.text().collect();
            let input_changes = {
                let text: String = input_changes.text().collect();
                let text = text.replace(&status, "");
                let texts: Vec<_> = text.trim().split_whitespace().collect();
                texts.join(" ")
            };

            let [succeeded, failed, queued, delta] = [succeeded, failed, queued, delta].map(|x| {
                let text: String = x.text().collect();
                text.trim().to_string()
            });

            let [succeeded, failed, queued]: [Result<u64, _>; 3] =
                [succeeded, failed, queued].map(|text| match text.is_empty() {
                    true => Ok(0),
                    false => text.parse(),
                });
            let delta = match delta {
                x if x.is_empty() => None,
                x => Some(x),
            };

            let finished = queued == Ok(0);
            let icon = match finished {
                true => StatusIcon::Succeeded,
                false => StatusIcon::Queued,
            };

            evals.push(EvalStatus {
                icon,
                finished: Some(finished),
                id: Some(id),
                url: Some(url.into()),
                datetime: Some(date.into()),
                relative: Some(relative),
                timestamp: Some(timestamp),
                status,
                short_rev: Some(short_rev),
                input_changes: Some(input_changes),
                succeeded: Some(succeeded?),
                failed: Some(failed?),
                queued: Some(queued?),
                delta,
            })
        }
        Ok(Self { evals, ..self })
    }

    pub fn fetch_and_format(self) -> anyhow::Result<String> {
        if self.args.url {
            return Ok(self.get_url().into());
        }
        let stat = self.fetch_and_parse()?;
        if stat.args.json {
            let json_string = serde_json::to_string_pretty(&stat.evals)?;
            return Ok(json_string);
        }
        let title = format!(
            "Evaluations of jobset {} {}",
            stat.args.jobset.bold(),
            format!("@ {}", stat.get_url()).dimmed()
        );
        let mut table = Table::new();
        table.load_preset(comfy_table::presets::NOTHING);
        for eval in stat.evals {
            table.add_row(eval.format_as_vec());
            if stat.args.short {
                break;
            }
        }
        for column in table.column_iter_mut() {
            column.set_padding((0, 1));
            // column.set_constraint(comfy_table::ColumnConstraint::ContentWidth);
            break; // only for the first column
        }
        Ok(format!("{}\n{}", title, table.trim_fmt()))
    }
}
