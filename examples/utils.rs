//! Facilities for generating hydra-check examples (with snapshots)

#![warn(missing_docs)]

/// Saves an inline [`insta`] snapshot for future use, returning the content
/// of the snapshot with horizontal whitespace trimmed.
/// It only calls [`insta::assert_snapshot!`] when `$do_assert = true`.
#[macro_export]
macro_rules! inline_snapshot {
    ($do_assert:expr, $($arg:expr),*, @$snapshot:literal $(,)?) => {
        {
            if $do_assert {
                insta::assert_snapshot!($($arg),*, @$snapshot);
            }
            $snapshot.trim().lines().map(|x| x.trim()).collect::<Vec<_>>().join("\n")
        }
    };
}

#[test]
fn macro_inline_snapshot() {
    let value = "alpha";

    // do assert
    let snapshot = inline_snapshot!(true, value, @"alpha");
    assert_eq!(snapshot, value);

    // don't assert
    let snapshot = inline_snapshot!(false, value, @"beta");
    assert_eq!(snapshot, "beta");
}

fn split_cli_args(args: &str) -> Box<[&str]> {
    args.split_whitespace().collect()
}

fn cmd(args: &str) -> duct::Expression {
    let all_args = split_cli_args(args);
    let mut args = all_args.iter();
    let program = *args.next().unwrap();
    duct::cmd(program, args)
}

/// Runs with `faketty`, depends on `faketty-run` under examples.
fn faketty_run(args: &str) -> anyhow::Result<String> {
    let args = format!(
        "cargo run --quiet --example=faketty-run -- {} {}",
        "cargo run --quiet --", args
    );
    Ok(cmd(args.trim()).stderr_to_stdout().read()?)
}

/// Runs hydra-check in a faketty and collect the colored console outputs
/// when `do_assert = true`, otherwise returns an empty string.
pub fn maybe_run_hydra_check(do_assert: bool, args: &str) -> anyhow::Result<String> {
    if do_assert {
        faketty_run(&args)
    } else {
        Ok("".into())
    }
}

/// Runs hydra-check with `$args` and saves its output as an inline [`insta`]
/// snapshot. When `do_assert = false`, simply prints the saved snapshot.
#[macro_export]
macro_rules! hydra_check {
    ($args:expr, do_assert = $do_assert:expr, @$snapshot:literal $(,)?) => {
        {
            let output = $crate::utils::maybe_run_hydra_check($do_assert, $args)?;
            let output = inline_snapshot!($do_assert, output, @$snapshot);
            println!("{}", output);
        }
    };
}
