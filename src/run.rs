use eyre::Result;
use std::{ffi::OsStr, path::PathBuf};

/// Invoke cargo check, but, set RUSTC_WORKSPACE_WRAPPER to this binary
pub fn cargo_check<T: AsRef<OsStr>>(
    command_mode: &'static str,
    command_selector: Option<String>,
    args: &[T],
) -> Result<()> {
    // FIXME: Is this the same cargo?
    tracing::debug!("cargo check is being called");

    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("check");
    cmd.args(args);
    cmd.env("RUSTC_WORKSPACE_WRAPPER", std::env::current_exe()?);
    cmd.env(crate::ENV_VAR_WHYNOT_MODE, command_mode);
    if let Some(selector) = command_selector {
        cmd.env(crate::ENV_VAR_WHYNOT_SELECTOR, selector);
    }
    cmd.stdout(std::process::Stdio::null());
    cmd.status()?;
    tracing::debug!("cargo check called");
    Ok(())
}

// runs rustc
pub fn rustc_run<C: rustc_driver::Callbacks + Send, T: AsRef<OsStr>>(
    callbacks: &mut C,
    args: &[T],
) -> Result<()> {
    let mut rustc_args = vec![String::from("rustc")];
    if let Ok(sysroot) = sysroot() {
        rustc_args.extend([
            String::from("--sysroot"),
            sysroot.to_string_lossy().to_string(),
        ])
    }
    rustc_args.extend(
        args.iter()
            .map(|s| s.as_ref().to_string_lossy().to_string()),
    );

    rustc_driver::RunCompiler::new(&rustc_args, callbacks)
        .run()
        .map_err(|_| std::process::exit(1))
}

fn sysroot() -> Result<PathBuf> {
    let rustup_home = std::env::var("RUSTUP_HOME")?;
    let rustup_toolchain = std::env::var("RUSTUP_TOOLCHAIN")?;
    Ok(PathBuf::from(rustup_home)
        .join("toolchains")
        .join(rustup_toolchain))
}
