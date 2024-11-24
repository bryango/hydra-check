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
#[allow(dead_code)]
fn faketty_run(args: &str) -> anyhow::Result<String> {
    let args = format!(
        "cargo run --quiet --example=faketty-run -- {} {}",
        "cargo run --quiet --", args
    );
    Ok(cmd(args.trim()).stderr_to_stdout().read()?)
}

/// Runs with [`portable_pty`], adapted from the examples provided in the
/// [`portable_pty`] crate.
#[allow(dead_code)]
fn pty_run(args: &str) -> anyhow::Result<String> {
    let args = format!("run --quiet -- {args}");
    let args = split_cli_args(&args);

    use portable_pty::{native_pty_system, CommandBuilder, PtySize};

    // Use the native pty implementation for the system
    let pty_system = native_pty_system();

    // Create a new pty
    let pair = pty_system.openpty(PtySize {
        rows: 24,
        cols: 80,
        // Not all systems support pixel_width, pixel_height,
        // but it is good practice to set it to something
        // that matches the size of the selected font.  That
        // is more complex than can be shown here in this
        // brief example though!
        pixel_width: 0,
        pixel_height: 0,
    })?;

    // Spawn a shell into the pty
    let mut cmd = CommandBuilder::new("cargo");
    cmd.cwd(env!("CARGO_MANIFEST_DIR"));
    cmd.args(args);
    let mut child = pair.slave.spawn_command(cmd)?;

    // Release any handles owned by the slave: we don't need it now
    // that we've spawned the child.
    drop(pair.slave);

    // Read the output in another thread.
    // This is important because it is easy to encounter a situation
    // where read/write buffers fill and block either your process
    // or the spawned process.
    let (tx, rx) = std::sync::mpsc::channel();
    let mut reader = pair.master.try_clone_reader()?;
    std::thread::spawn(move || {
        // Consume the output from the child
        let mut buf = String::new();
        reader.read_to_string(&mut buf).unwrap();
        tx.send(buf).unwrap();
    });

    {
        // Obtain the writer.
        // When the writer is dropped, EOF will be sent to
        // the program that was spawned.
        // It is important to take the writer even if you don't
        // send anything to its stdin so that EOF can be
        // generated, otherwise you risk deadlocking yourself.
        let mut writer = pair.master.take_writer()?;

        // This example doesn't need to write anything, but if you
        // want to send data to the child, you'd set `to_write` to
        // that data and do it like this:
        let to_write = "";
        if !to_write.is_empty() {
            // To avoid deadlock, wrt. reading and waiting, we send
            // data to the stdin of the child in a different thread.
            std::thread::spawn(move || {
                writer.write_all(to_write.as_bytes()).unwrap();
            });
        }
    }

    // Wait for the child to complete
    println!("child status: {:?}", child.wait()?);

    // Take care to drop the master after our processes are
    // done, as some platforms get unhappy if it is dropped
    // sooner than that.
    drop(pair.master);

    // Now wait for the output to be read by our reader thread
    Ok(rx.recv()?)
}

/// Runs hydra-check in a faketty and collect the colored console outputs
/// when `do_assert = true`, otherwise returns an empty string.
pub fn maybe_run_hydra_check(do_assert: bool, args: &str) -> anyhow::Result<String> {
    let faketty_run = pty_run;
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
