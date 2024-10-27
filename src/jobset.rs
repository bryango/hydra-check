use anyhow::bail;
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
    fn jobset(&self) -> &str {
        self.args.jobset.as_str()
    }

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
                delta: Some(delta.into()),
            })
        }
        Ok(Self { evals, ..self })
    }
}
