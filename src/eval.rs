use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{args::Evaluation, builds::BuildStatus, FetchHydra, FormatVecColored, StatusIcon};

#[skip_serializing_none]
#[derive(Serialize, Clone, Default)]
struct EvalInput {
    name: String,
    #[serde(rename = "type")]
    input_type: String,
    value: String,
    revision: String,
    store_path: String,
}

impl FormatVecColored for EvalInput {
    fn format_as_vec(&self) -> Vec<colored::ColoredString> {
        todo!()
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
    type Status = EvalInput;

    fn name(&self) -> &str {
        self.eval.spec.as_str()
    }

    fn get_url(&self) -> &str {
        &self.url
    }

    fn entries(&self) -> &Vec<Self::Status> {
        &self.inputs
    }

    fn finish_with_error(self, status: String) -> Self {
        Self {
            inputs: vec![EvalInput {
                name: StatusIcon::Warning.to_string(),
                value: status,
                ..Default::default()
            }],
            ..self
        }
    }
}
