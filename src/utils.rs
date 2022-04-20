pub fn install_utils() -> eyre::Result<()> {
    install_tracing();
    install_eyre()?;
    Ok(())
}

fn install_eyre() -> eyre::Result<()> {
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default().into_hooks();

    eyre_hook.install()?;

    std::panic::set_hook(Box::new(move |pi| {
        tracing::error!("{}", panic_hook.panic_report(pi));
    }));
    Ok(())
}

fn install_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer().with_writer(std::io::stderr).with_target(true);
    #[rustfmt::skip]
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .map(|f| {
            f.add_directive("rustc=error".parse().expect("could not make directive"))
        })
        .expect("could not make filter layer");

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}
