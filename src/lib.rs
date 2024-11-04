mod args;
pub mod constants;
mod fetch_stable;
mod queries;
mod soup;
mod structs;

use std::time::Duration;

pub use args::Cli;
use args::ResolvedArgs;
use colored::{ColoredString, Colorize};
use comfy_table::Table;
pub use fetch_stable::NixpkgsChannelVersion;
use scraper::{ElementRef, Html};
pub use soup::{SoupFind, TryAttr};
use structs::{BuildStatus, Evaluation, StatusIcon};

trait FormatVecColored {
    fn format_as_vec(&self) -> Vec<ColoredString>;
}

trait FetchHydra: Sized + Clone {
    fn get_url(&self) -> &str;
    fn fetch_document(&self) -> anyhow::Result<Html> {
        let document = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
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
                    format!(
                        "Unknown Hydra Error with '{}' found at {}",
                        selectors,
                        self.get_url()
                    )
                };
                // sanitize the text a little bit
                let status: Vec<&str> = status.lines().map(str::trim).collect();
                let status: String = status.join(" ");
                Err(self.clone().finish_with_error(status))
            }
            Ok(tbody) => Ok(tbody),
        }
    }

    fn format_table<T: FormatVecColored>(&self, short: bool, entries: &Vec<T>) -> String {
        let mut table = Table::new();
        table.load_preset(comfy_table::presets::NOTHING);
        // .set_content_arrangement(comfy_table::ContentArrangement::Dynamic);
        for entry in entries {
            table.add_row(entry.format_as_vec());
            if short {
                break;
            }
        }
        for (idx, column) in table.column_iter_mut().enumerate() {
            if idx == 0 {
                column.set_padding((0, 1));
            }
            // column.set_constraint(comfy_table::ColumnConstraint::ContentWidth);
        }
        table.trim_fmt()
    }
}

fn is_skipable_row(row: ElementRef<'_>) -> anyhow::Result<bool> {
    let link = row.find("td")?.find("a")?.try_attr("href")?;
    let skipable = link.ends_with("/all") || link.contains("full=1");
    Ok(skipable)
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
