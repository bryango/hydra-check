mod args;
mod fetch_stable;
mod soup;

pub use args::Args;
pub use fetch_stable::NixpkgsChannelVersion;
pub use soup::SoupFind;

pub fn log_format(
    w: &mut dyn std::io::Write,
    _now: &mut flexi_logger::DeferredNow,
    record: &log::Record,
) -> Result<(), std::io::Error> {
    write!(
        w,
        "{}: {}",
        record.level().as_str().to_lowercase(),
        &record.args()
    )
}
