use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{args::Evaluation, builds::BuildStatus};

#[skip_serializing_none]
#[derive(Serialize)]
struct EvalInput {
    name: String,
    #[serde(rename = "type")]
    input_type: String,
    value: String,
    revision: String,
    store_path: String,
}

#[skip_serializing_none]
#[derive(Serialize)]
struct EvalInputChanges {
    input: String,
    description: String,
    url: String,
    revs: (String, String),
    short_revs: (String, String),
}

#[derive(Serialize)]
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
