//! Run command in a fake tty, preserving colored outputs.
//! Ported from <https://github.com/dtolnay/faketty>.

mod faketty;

use faketty::Error;
use std::ffi::CString;

fn main() -> Result<(), Error> {
    let args: Vec<CString> = std::env::args()
        .into_iter()
        .skip(1)
        .map(|x| CString::new(x.as_bytes()).unwrap())
        .collect();
    if args.is_empty() {
        eprintln!("fatal: no program supplied");
        std::process::exit(1);
    };
    let _ = faketty::run_command(args)?;
    Ok(())
}
