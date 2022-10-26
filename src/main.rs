//! See `whynot.rs` for the main functionality of this binary
#![feature(rustc_private)]

use color_eyre::Help;

fn main() -> Result<std::process::ExitCode, color_eyre::Report> {
    color_eyre::install()?;
    let path = std::process::Command::new("rustc")
        .args(["--print", "sysroot"])
        .output()?
        .stdout;
    let status = std::process::Command::new("whynot")
        .args(std::env::args_os().skip(1))
        .env("LD_LIBRARY_PATH", String::from_utf8(path)?)
        .status()?;

    if !status.success() {
        return Err(eyre::eyre!("couldnt run proxy")).with_note(|| "if this failed to load the rustc_driver library, try reinstalling `cargo-whynot` with `cargo +nightly install --force cargo-whynot`");
    }
    Ok(0.into())
}
