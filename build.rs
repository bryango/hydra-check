use git2::{DescribeFormatOptions, DescribeOptions};

fn main() -> Result<(), git2::Error> {
    let Ok(repo) = git2::Repository::open_from_env() else {
        // not in a git repository, probably building from a release tarball
        return Ok(());
    };
    let dirty_suffix = "-dirty";

    // run: git describe --tags --abbrev=7 --dirty
    // we do not use `--long` up front in case we are exactly at a release tag
    let mut desc_opts = DescribeOptions::new();
    let description = repo.describe(&desc_opts.describe_tags())?;

    let mut fmt_opts = DescribeFormatOptions::new();
    fmt_opts.abbreviated_size(7).dirty_suffix(&dirty_suffix);

    let describe_format = |fmt_opts: &DescribeFormatOptions| -> Result<String, git2::Error> {
        let desc = description
            .format(Some(fmt_opts))?
            .trim_start_matches("v")
            .to_string();
        Ok(desc)
    };
    let desc = describe_format(&fmt_opts)?;

    let pkg_version = env!("CARGO_PKG_VERSION");
    let desc = if !desc.starts_with(pkg_version) {
        println!("cargo::warning=inconsistent versioning: package={pkg_version} vs git={desc}");
        // rerun: git describe --long
        fmt_opts.always_use_long_format(true);
        describe_format(&fmt_opts)?
    } else {
        desc.to_string()
    };

    let version = match desc.rsplit_once("-g") {
        Some((_, short_rev)) => format!("{pkg_version}-g{short_rev}"),
        None => format!(
            "{pkg_version}{}",
            match desc.ends_with(dirty_suffix) {
                true => dirty_suffix,
                false => "",
            }
        ),
    };
    println!("cargo::rustc-env=CARGO_PKG_VERSION={version}");
    println!("cargo:rerun-if-changed={}", repo.path().to_string_lossy());
    Ok(())
}
