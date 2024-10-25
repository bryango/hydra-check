mod args;
mod build;
pub mod constants;
mod fetch_stable;
mod soup;

pub use args::Args;
use args::ResolvedArgs;
pub use build::PackageStatus;
use colored::Colorize;
use fetch_stable::NixpkgsChannelVersion;
pub use soup::{SoupFind, TryAttr};

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
