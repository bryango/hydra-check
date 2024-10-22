use std::fmt::Display;

use serde::Serialize;
use serde_with::{skip_serializing_none, SerializeDisplay};

#[skip_serializing_none]
#[derive(Serialize, Debug)]
struct BuildStatus {
    icon: StatusIcon,
    success: bool,
    status: String,
    timestamp: String,
    build_id: String,
    build_url: String,
    name: String,
    arch: String,
    evals: bool,
}

#[derive(SerializeDisplay, Debug, Clone)]
enum StatusIcon {
    Success,
    Failure,
    Unknown,
}

impl Display for StatusIcon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let icon = match self {
            Self::Success => "✔",
            Self::Failure => "✖",
            Self::Unknown => "⚠",
        };
        write!(f, "{icon}")
    }
}

#[test]
fn serialize_success_icon() {
    let success_icon = serde_json::to_string(&StatusIcon::Success).unwrap();
    debug_assert_eq!(success_icon, r#""✔""#)
}
