mod args;
pub mod constants;
mod fetch_stable;
pub mod parse;
mod soup;

pub use args::Args;
use colored::Colorize;
pub use fetch_stable::NixpkgsChannelVersion;
pub use soup::SoupFind;

pub fn log_format(
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
