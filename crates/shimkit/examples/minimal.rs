use shimkit::args::Arguments;
use shimkit::event::EventPublisher;
use shimkit::utils::cri_sandbox_id;
use shimkit_types::sandbox::Sandbox;
use shimkit_types::task::{Task, VersionResponse};
use tokio::signal::ctrl_c;
use trapeze::Result;

struct Server {
    _publisher: EventPublisher,
}

impl Task for Server {
    // implement the trait functions

    async fn version(&self, _: ()) -> Result<VersionResponse> {
        Ok(VersionResponse {
            executable: env!("CARGO_BIN_NAME").into(),
            info: vec![("Version", env!("CARGO_PKG_VERSION")).into()],
        })
    }
}

impl Sandbox for Server {
    // implement the trait functions
}

#[shimkit::main]
async fn main(args: Arguments) -> Result<()> {
    env_logger::init();

    let address = if args.is_interactive() {
        log::info!("Running shim interactively, a debug address will be used");
        args.socket_address_debug("debug")
    } else if let Some(id) = cri_sandbox_id() {
        args.socket_address(&id)
    } else {
        args.socket_address(&args.id)
    };

    #[cfg(unix)]
    let _ = tokio::fs::remove_file(&address).await;

    let _publisher = args.event_publisher().await?;
    let server = Server { _publisher };
    let handle = args.serve(&address, server).await?;

    log::info!("Listening on {}", address.display());
    log::info!("Press Ctrl+C to exit.");

    let controller = handle.controller();
    tokio::spawn(async move {
        if ctrl_c().await.is_err() {
            log::error!("Failed to wait for Ctrl+C.");
        }
        log::info!("Shutting down server");
        controller.shutdown();
    });

    handle.await.expect("Error shutting down server");

    Ok(())
}
