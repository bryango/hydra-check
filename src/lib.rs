mod args;
mod builds;
pub mod constants;
mod fetch_stable;
mod jobset;
mod soup;

use std::time::Duration;

pub use args::Args;
pub use args::{Queries, ResolvedArgs};
pub use builds::{BuildStatus, PackageStatus};
use colored::{ColoredString, Colorize};
use fetch_stable::NixpkgsChannelVersion;
use scraper::{ElementRef, Html};
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

trait FetchData: Sized {
    fn get_url(&self) -> &str;
    fn fetch_document(&self) -> anyhow::Result<Html> {
        let document = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()?
            .get(self.get_url())
            .send()?
            .error_for_status()?
            .text()?;
        Ok(Html::parse_document(&document))
    }

    /// Checks if the fetched [Html] contains a `tbody` tag (table body).
    /// If not, returns the alert text. If yes, returns the found element.
    fn find_tbody<'a>(&self, doc: &'a Html) -> Result<ElementRef<'a>, String> {
        match doc.find("tbody") {
            Err(_) => {
                // either the package was not evaluated (due to being e.g. unfree)
                // or the package does not exist
                let status = if let Ok(alert) = doc.find("div.alert") {
                    alert.text().collect()
                } else {
                    format!("Unknown Hydra Error found at {}", self.get_url())
                };
                // sanitize the text a little bit
                let status: Vec<&str> = status.lines().map(str::trim).collect();
                let status: String = status.join(" ");
                Err(status)
            }
            Ok(tbody) => Ok(tbody),
        }
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
