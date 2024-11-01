mod args;
mod builds;
pub mod constants;
mod eval;
mod fetch_stable;
mod jobset;
mod soup;

use std::time::Duration;

pub use args::Cli;
use args::ResolvedArgs;
use builds::BuildStatus;
use colored::{ColoredString, Colorize};
use comfy_table::Table;
pub use fetch_stable::NixpkgsChannelVersion;
use scraper::{ElementRef, Html};
use serde_with::SerializeDisplay;
pub use soup::{SoupFind, TryAttr};

#[derive(SerializeDisplay, Debug, Clone, Default)]
enum StatusIcon {
    Succeeded,
    Failed,
    Cancelled,
    Queued,
    #[default]
    Warning,
}

impl From<&StatusIcon> for ColoredString {
    fn from(icon: &StatusIcon) -> Self {
        match icon {
            StatusIcon::Succeeded => "✔".green(),
            StatusIcon::Failed => "✖".red(),
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

trait FormatVecColored {
    fn format_as_vec(&self) -> Vec<ColoredString>;
}

trait FetchHydra: Sized + Clone {
    type Status: FormatVecColored;

    fn get_url(&self) -> &str;
    fn entries(&self) -> &Vec<Self::Status>;
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

    fn finish_with_error(self, status: String) -> Self;

    /// Checks if the fetched [Html] contains a `tbody` tag (table body).
    /// If not, returns the alert text. If yes, returns the found element.
    fn find_tbody<'a>(&self, doc: &'a Html, selector: &str) -> Result<ElementRef<'a>, Self> {
        let selectors = format!("{} tbody", selector);
        match doc.find(selectors.trim()) {
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
                Err(self.clone().finish_with_error(status))
            }
            Ok(tbody) => Ok(tbody),
        }
    }

    fn is_skipable_row(row: ElementRef<'_>) -> anyhow::Result<bool> {
        let skipable = row
            .find("td")?
            .find("a")?
            .try_attr("href")?
            .ends_with("/all");
        Ok(skipable)
    }

    fn format_table(&self, short: bool) -> String {
        let mut table = Table::new();
        table.load_preset(comfy_table::presets::NOTHING);
        for entry in self.entries() {
            table.add_row(entry.format_as_vec());
            if short {
                break;
            }
        }
        for column in table.column_iter_mut() {
            column.set_padding((0, 1));
            // column.set_constraint(comfy_table::ColumnConstraint::ContentWidth);
            break; // only for the first column
        }
        table.trim_fmt()
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
    let success_icon = serde_json::to_string(&StatusIcon::Succeeded).unwrap();
    debug_assert_eq!(success_icon, r#""✔""#)
}
