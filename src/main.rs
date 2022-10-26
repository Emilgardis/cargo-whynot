//! See `whynot.rs` for the main functionality of this binary
#![feature(rustc_private)]

fn main() -> Result<std::process::ExitCode, color_eyre::Report> {
    color_eyre::install()?;
    let path = std::process::Command::new("rustc")
        .args(["--print", "sysroot"])
        .output()?
        .stdout;
    std::process::Command::new("whynot")
        .args(std::env::args_os().skip(1))
        .env("LD_LIBRARY_PATH", String::from_utf8(path)?)
        .status()?;
    Ok(std::process::ExitCode::SUCCESS)
}
