use shimkit::types::cri::*;
use shimkit::types::sandbox::*;
use shimkit::types::{Result, Status};

use super::Server;

impl Sandbox for Server {
    async fn create_sandbox(&self, mut r: CreateSandboxRequest) -> Result<CreateSandboxResponse> {
        let options = r.options.take().and_then(|mut options| {
            if !options.type_url.ends_with("runtime.v1.PodSandboxConfig") {
                return None;
            }
            // Someone somewhere is not adding the required slash to the type_url
            // Workaround it by setting it manually
            options.type_url = "/runtime.v1.PodSandboxConfig".into();
            options.to_msg::<PodSandboxConfig>().ok()
        });
        log::info!("{r:#?}");
        options.inspect(|opts| log::info!("{opts:#?}"));
        Err(Status::not_found(
            "/containerd.runtime.sandbox.v1.Sandbox/CreateSandbox is not supported",
        ))
    }

    async fn ping_sandbox(&self, r: PingRequest) -> Result<PingResponse> {
        log::info!("{r:#?}");
        Ok(PingResponse {})
    }

    async fn platform(&self, r: PlatformRequest) -> Result<PlatformResponse> {
        log::info!("{r:#?}");
        Err(Status::not_found(
            "/containerd.runtime.sandbox.v1.Sandbox/Platform is not supported",
        ))
    }

    async fn sandbox_status(&self, r: SandboxStatusRequest) -> Result<SandboxStatusResponse> {
        log::info!("{r:#?}");
        Err(Status::not_found(
            "/containerd.runtime.sandbox.v1.Sandbox/SandboxStatus is not supported",
        ))
    }

    async fn shutdown_sandbox(&self, r: ShutdownSandboxRequest) -> Result<ShutdownSandboxResponse> {
        log::info!("{r:#?}");
        Err(Status::not_found(
            "/containerd.runtime.sandbox.v1.Sandbox/ShutdownSandbox is not supported",
        ))
    }

    async fn start_sandbox(&self, r: StartSandboxRequest) -> Result<StartSandboxResponse> {
        log::info!("{r:#?}");
        Err(Status::not_found(
            "/containerd.runtime.sandbox.v1.Sandbox/StartSandbox is not supported",
        ))
    }

    async fn stop_sandbox(&self, r: StopSandboxRequest) -> Result<StopSandboxResponse> {
        log::info!("{r:#?}");
        Err(Status::not_found(
            "/containerd.runtime.sandbox.v1.Sandbox/StopSandbox is not supported",
        ))
    }

    async fn wait_sandbox(&self, r: WaitSandboxRequest) -> Result<WaitSandboxResponse> {
        log::info!("{r:#?}");
        Err(Status::not_found(
            "/containerd.runtime.sandbox.v1.Sandbox/WaitSandbox is not supported",
        ))
    }
}
