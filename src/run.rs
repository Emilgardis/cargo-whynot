use eyre::{Context, Result};
use rustc_codegen_ssa::traits::CodegenBackend;
use rustc_session::config;
use std::{ffi::OsStr, path::PathBuf};

/// Invoke cargo check, but, set RUSTC_WORKSPACE_WRAPPER to this binary
pub fn cargo_check<T: AsRef<OsStr>>(
    command_mode: &'static str,
    command_selector: Option<String>,
    package: &Option<String>,
    rustflags: Option<&'static str>,
    args: &[T],
) -> Result<()> {
    // FIXME: Is this the same cargo?
    tracing::debug!("cargo check is being called");

    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("check");
    //cmd.arg("-q");
    cmd.args(args);
    if let Some(package) = package {
        cmd.args(["-p", package]);
    }

    cmd.env("RUSTC_WORKSPACE_WRAPPER", std::env::current_exe()?);
    cmd.env(crate::ENV_VAR_WHYNOT_MODE, command_mode);
    if let Some(flags) = rustflags {
        cmd.env("RUSTFLAGS", flags);
    }
    if let Some(selector) = command_selector {
        cmd.env(crate::ENV_VAR_WHYNOT_SELECTOR, selector);
    }
    // cmd.stdout(std::process::Stdio::null());
    cmd.status()?;
    Ok(())
}

// runs rustc
#[allow(clippy::type_complexity)]
pub fn rustc_run<T: AsRef<OsStr>>(
    callbacks: Option<&mut (dyn rustc_driver::Callbacks + Send)>,
    set_codegen: Option<Box<dyn FnOnce(&config::Options) -> Box<dyn CodegenBackend> + Send>>,
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

    let cb = if let Some(callbacks) = callbacks {
        callbacks
    } else {
        #[allow(clippy::box_default)]
        Box::leak(Box::new(rustc_driver::TimePassesCallbacks::default()))
    };

    let mut run = rustc_driver::RunCompiler::new(&rustc_args, cb);
    run.set_make_codegen_backend(set_codegen);
    run.run().map_err(|e| eyre::eyre!("error: {e:?}"))
}

fn sysroot() -> Result<PathBuf> {
    let rustup_home = std::env::var("RUSTUP_HOME")?;
    let rustup_toolchain = std::env::var("RUSTUP_TOOLCHAIN")?;
    Ok(PathBuf::from(rustup_home)
        .join("toolchains")
        .join(rustup_toolchain))
}
