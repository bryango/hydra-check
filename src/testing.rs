//! Testing facilities for internal use only

#[allow(rustdoc::broken_intra_doc_links)]
/// Saves an inline [`insta`] snapshot for future use, returning the content
/// of the snapshot. Only calls [`insta::assert_snapshot!`]
/// when `do_assert = true` _and_ in `#[cfg(test)]` mode.
/// ```no
/// use pacjump::testing::inline_snapshot;
/// let value = "alpha";
///
/// // do assert
/// inline_snapshot!(do_assert = true, value, @"alpha");
/// inline_snapshot!(do_assert = true, value, @"beta").unwrap_err();
///
/// // don't assert
/// let snapshot = inline_snapshot!(do_assert = false, value, @"beta");
/// assert_eq!(snapshot, "beta");
/// ```
#[macro_export]
#[doc(hidden)] // don't show at the crate root, re-export later
macro_rules! inline_snapshot {
    (do_assert = $do_assert:literal, $($arg:expr),*, @$snapshot:literal $(,)?) => {
        {
            #[cfg(test)]
            if $do_assert {
                insta::assert_snapshot!($($arg),*, @$snapshot);
            }
            $snapshot
        }
    };
}

#[doc(inline)]
pub use inline_snapshot;

#[cfg(test)]
fn split_cli_args(args: &str) -> Box<[&str]> {
    args.split_whitespace().collect()
}

#[cfg(test)]
fn cmd<T>(args: T) -> duct::Expression
where
    T: AsRef<str>,
{
    fn inner(args: &str) -> duct::Expression {
        let all_args = split_cli_args(args);
        let mut args = all_args.iter();
        let program = *args.next().unwrap();
        duct::cmd(program, args)
    }
    inner(args.as_ref())
}

#[cfg(test)]
pub fn cargo_run<T>(args: T) -> duct::Expression
where
    T: AsRef<str>,
{
    fn inner(args: &str) -> duct::Expression {
        let args = format!("cargo run -- {args}");
        cmd(args.trim())
    }
    inner(args.as_ref())
}
