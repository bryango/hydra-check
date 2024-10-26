mod args;
mod builds;
pub mod constants;
mod fetch_stable;
mod soup;

pub use args::Args;
pub use args::{Queries, ResolvedArgs};
pub use builds::{BuildStatus, PackageStatus};
use colored::{ColoredString, Colorize};
use fetch_stable::NixpkgsChannelVersion;
use serde_with::SerializeDisplay;
pub use soup::{SoupFind, TryAttr};

#[derive(SerializeDisplay, Debug, Clone, Default)]
enum StatusIcon {
    Success,
    Failure,
    Cancelled,
    Queued,
    #[default]
    Warning,
}

impl From<&StatusIcon> for ColoredString {
    fn from(icon: &StatusIcon) -> Self {
        match icon {
            StatusIcon::Success => "✔".green(),
            StatusIcon::Failure => "✖".red(),
            StatusIcon::Cancelled => "⏹".red(),
            StatusIcon::Queued => "⧖".yellow(),
            StatusIcon::Warning => "⚠".yellow(),
        }
    }
}

impl std::fmt::Display for StatusIcon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let icon = ColoredString::from(self).normal();
        write!(f, "{icon}")
    }
}

fn log_format(
    w: &mut dyn std::io::Write,
    _now: &mut flexi_logger::DeferredNow,
    record: &log::Record,
) -> Result<(), std::io::Error> {
    let level = record.level();
    let color = match level {
        log::Level::Error => "red",
        log::Level::Warn => "yellow",
        _ => "",
    };
    let level = format!("{level}:").to_lowercase().color(color).bold();
    write!(w, "{} {}", level, &record.args())
}

#[test]
fn serialize_success_icon() {
    let success_icon = serde_json::to_string(&StatusIcon::Success).unwrap();
    debug_assert_eq!(success_icon, r#""✔""#)
}
