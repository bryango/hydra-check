use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{ResolvedArgs, StatusIcon};

#[skip_serializing_none]
#[derive(Serialize, Debug, Default)]
/// Status of a single evaluation, can be serialized a JSON entry
pub struct EvalStatus {
    icon: StatusIcon,
    finished: bool,
    id: u64,
    url: String,
    datetime: String,
    relative: String,
    timestamp: u64,
    status: String,
    short_rev: String,
    input_changes: String,
    succeeded: u64,
    failed: u64,
    queued: u64,
    delta: String,
}

/// Container for the eval status and metadata of a jobset
pub struct JobsetStatus<'a> {
    args: &'a ResolvedArgs,
    url: String,
    /// Status of recent evaluations of the jobset
    pub builds: Vec<EvalStatus>,
}

impl<'a> JobsetStatus<'a> {}
