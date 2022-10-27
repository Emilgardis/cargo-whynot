//! See `whynot.rs` for the main functionality of this binary
#![feature(rustc_private)]

use color_eyre::Help;
mod utils;

fn main() -> Result<(), color_eyre::Report> {
    utils::install_utils()?;
    let path = std::process::Command::new("rustc")
        .args(["--print", "sysroot"])
        .output()?
        .stdout;
    let status = std::process::Command::new("whynot")
        .args(std::env::args_os().skip(1))
        .env("LD_LIBRARY_PATH", String::from_utf8(path)?.trim())
        .status()?;

    if !status.success() {
        if !matches!(status.code(), Some(101)) {
            #[cfg(target_family = "unix")]
            {
                use std::os::unix::process::ExitStatusExt;
                if status.signal() == Some(6) {
                    return Err(eyre::eyre!("couldnt run proxy")).with_note(|| "if this failed to load the rustc_driver library, try reinstalling `cargo-whynot` with `cargo +nightly install --force cargo-whynot`");
                }
            }
        }
        std::process::exit(status.code().unwrap_or(1))
    }

    Ok(())
}
