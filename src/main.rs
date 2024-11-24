use hydra_check::HydraCheckCli;

fn main() -> anyhow::Result<()> {
    let success = HydraCheckCli::execute()?;
    if !success {
        std::process::exit(1);
    }
    Ok(())
}

/// Updates console examples in README automatically,
/// if the environment variable `TRYCMD=overwrite`.
#[test]
#[ignore = "require internet connection, and not reproducible"]
fn trycmd() {
    trycmd::TestCases::new().case("README.md");
}

#[cfg(test)]
use term_transcript::{
    svg::{NamedPalette, Template, TemplateOptions},
    traits::ConfigureCommand,
    PtyCommand, ShellOptions, Transcript, UserInput,
};

#[test]
fn write_transcript() -> anyhow::Result<()> {
    use std::str::FromStr;

    let inputs = vec![UserInput::command(r#"hydra-check"#)];

    let command = {
        let mut command = PtyCommand::default();
        let dir = std::path::PathBuf::from_str(env!("CARGO_MANIFEST_DIR"))?;
        command.current_dir(&dir);
        command
    };
    let mut options = ShellOptions::new(command)
        .with_cargo_path()
        .with_io_timeout(std::time::Duration::from_secs(10));
    let transcript = Transcript::from_inputs(&mut options, inputs)?;

    let mut file = std::fs::File::create("stdout.term.svg")?;
    Template::new(TemplateOptions {
        width: 1024,
        palette: NamedPalette::Ubuntu.into(),
        wrap: None,
        ..TemplateOptions::default()
    })
    .render(&transcript, &mut file)?;
    file.sync_all()?;
    Ok(())
}
