//! See `whynot.rs` for the main functionality of this binary
#![feature(rustc_private)]

fn main() -> Result<std::process::ExitCode, color_eyre::Report> {
    std::process::Command::new("whynot")
        .args(std::env::args_os())
        .status()?;
    Ok(std::process::ExitCode::SUCCESS)
}
