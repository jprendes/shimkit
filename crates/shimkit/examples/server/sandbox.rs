use shimkit::protos::containerd::runtime::sandbox::v1::*;
use trapeze::{Result, Status};

use super::Server;

impl Sandbox for Server {
    async fn create_sandbox(&self, r: CreateSandboxRequest) -> Result<CreateSandboxResponse> {
        println!("{r:#?}");
        Err(Status::not_found(
            "/containerd.runtime.sandbox.v1.Sandbox/CreateSandbox is not supported",
        ))
    }

    async fn ping_sandbox(&self, r: PingRequest) -> Result<PingResponse> {
        println!("{r:#?}");
        Ok(PingResponse {})
    }

    async fn platform(&self, r: PlatformRequest) -> Result<PlatformResponse> {
        println!("{r:#?}");
        Err(Status::not_found(
            "/containerd.runtime.sandbox.v1.Sandbox/Platform is not supported",
        ))
    }

    async fn sandbox_status(&self, r: SandboxStatusRequest) -> Result<SandboxStatusResponse> {
        println!("{r:#?}");
        Err(Status::not_found(
            "/containerd.runtime.sandbox.v1.Sandbox/SandboxStatus is not supported",
        ))
    }

    async fn shutdown_sandbox(&self, r: ShutdownSandboxRequest) -> Result<ShutdownSandboxResponse> {
        println!("{r:#?}");
        Err(Status::not_found(
            "/containerd.runtime.sandbox.v1.Sandbox/ShutdownSandbox is not supported",
        ))
    }

    async fn start_sandbox(&self, r: StartSandboxRequest) -> Result<StartSandboxResponse> {
        println!("{r:#?}");
        Err(Status::not_found(
            "/containerd.runtime.sandbox.v1.Sandbox/StartSandbox is not supported",
        ))
    }

    async fn stop_sandbox(&self, r: StopSandboxRequest) -> Result<StopSandboxResponse> {
        println!("{r:#?}");
        Err(Status::not_found(
            "/containerd.runtime.sandbox.v1.Sandbox/StopSandbox is not supported",
        ))
    }

    async fn wait_sandbox(&self, r: WaitSandboxRequest) -> Result<WaitSandboxResponse> {
        println!("{r:#?}");
        Err(Status::not_found(
            "/containerd.runtime.sandbox.v1.Sandbox/WaitSandbox is not supported",
        ))
    }
}
