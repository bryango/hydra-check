use std::fmt::Display;

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
