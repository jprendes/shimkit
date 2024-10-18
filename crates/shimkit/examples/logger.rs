use std::io::Write;
use std::path::PathBuf;

use shimkit::args::Command;
use shimkit::utils::{cri_sandbox_id, serve};
use tokio::signal::ctrl_c;

mod server;
use server::Server;

#[shimkit::main(flavor = "current_thread")]
async fn main(cmd: Command) {
    match cmd {
        Command::Version => {
            let os_args: Vec<_> = std::env::args_os().collect();
            let argv0 = PathBuf::from(&os_args[0]);
            let argv0 = argv0.file_stem().unwrap_or_default().to_string_lossy();

            println!("{argv0}:");
            println!("  Runtime: {}", "logger");
            println!("  Version: {}", "0.1.0");
            println!("  Revision: {}", "<none>");
            println!();
        }
        Command::Start { mut pipe, args } => {
            let id = cri_sandbox_id().unwrap_or_else(|| args.id.clone());
            let address = args.socket_address(id);
            #[cfg(unix)]
            let _ = tokio::fs::remove_file(&address).await;

            let server = serve(&address, Server).await.unwrap();

            writeln!(pipe, "unix://{}", address.display()).unwrap();
            drop(pipe);

            println!("Listening on {}", address.display());
            println!("Press Ctrl+C to exit.");

            ctrl_c().await.expect("Failed to wait for Ctrl+C.");
            println!();
            println!("Shutting down server");

            server.shutdown();
            server.await.expect("Error shutting down server");
        }
        Command::Delete { bundle: _, args: _ } => {}
    }
}
