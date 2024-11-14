use shimkit::args::Arguments;
use shimkit::utils::cri_sandbox_id;
use tokio::signal::ctrl_c;
use anyhow::Result;

mod server;
use server::Server;

#[shimkit::main(flavor = "current_thread")]
async fn main(args: Arguments) -> Result<()> {
    env_logger::init();

    let address = if args.is_interactive() {
        log::info!("Running logger interactively, a debug address will be used");
        args.socket_address_debug("debug")
    } else {
        let id = cri_sandbox_id().unwrap_or_else(|| args.id.clone());
        args.socket_address(id)
    };

    #[cfg(unix)]
    let _ = tokio::fs::remove_file(&address).await;

    let _events = args.event_publisher().await?;

    let server = args.serve(&address, Server).await?;

    log::info!("Listening on {}", address.display());
    log::info!("Press Ctrl+C to exit.");

    let controller = server.controller();
    tokio::spawn(async move {
        ctrl_c().await.expect("Failed to wait for Ctrl+C.");
        log::info!("");
        log::info!("Shutting down server");
        controller.shutdown();
    });

    server.await.expect("Error shutting down server");
    log::info!("Server shutdown");

    Ok(())
}
