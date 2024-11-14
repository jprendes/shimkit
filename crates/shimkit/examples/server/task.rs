use std::env::current_exe;
use std::time::SystemTime;

use anyhow::Context;
use shimkit::types::task::*;
use shimkit::types::{Result, Status};

use super::Server;

impl Task for Server {
    async fn checkpoint(&self, r: CheckpointTaskRequest) -> Result<()> {
        log::info!("{r:#?}");
        Ok(())
    }

    async fn close_io(&self, r: CloseIoRequest) -> Result<()> {
        log::info!("{r:#?}");
        Ok(())
    }

    async fn connect(&self, r: ConnectRequest) -> Result<ConnectResponse> {
        log::info!("{r:#?}");
        Err(Status::not_found(
            "/containerd.task.v2.Task/Connect is not supported",
        ))
    }

    async fn create(&self, r: CreateTaskRequest) -> Result<CreateTaskResponse> {
        log::info!("{r:#?}");
        Err(Status::not_found(
            "/containerd.task.v2.Task/Create is not supported",
        ))
    }

    async fn delete(&self, r: DeleteRequest) -> Result<DeleteResponse> {
        log::info!("{r:#?}");
        Err(Status::not_found(
            "/containerd.task.v2.Task/Delete is not supported",
        ))
    }

    async fn exec(&self, r: ExecProcessRequest) -> Result<()> {
        log::info!("{r:#?}");
        Ok(())
    }

    async fn kill(&self, r: KillRequest) -> Result<()> {
        log::info!("{r:#?}");
        Ok(())
    }

    async fn pause(&self, r: PauseRequest) -> Result<()> {
        log::info!("{r:#?}");
        Ok(())
    }

    async fn pids(&self, r: PidsRequest) -> Result<PidsResponse> {
        log::info!("{r:#?}");
        Err(Status::not_found(
            "/containerd.task.v2.Task/Pids is not supported",
        ))
    }

    async fn resize_pty(&self, r: ResizePtyRequest) -> Result<()> {
        log::info!("{r:#?}");
        Ok(())
    }

    async fn resume(&self, r: ResumeRequest) -> Result<()> {
        log::info!("{r:#?}");
        Ok(())
    }

    async fn shutdown(&self, r: ShutdownRequest) -> Result<()> {
        log::info!("{r:#?}");
        Ok(())
    }

    async fn start(&self, r: StartRequest) -> Result<StartResponse> {
        log::info!("{r:#?}");
        Err(Status::not_found(
            "/containerd.task.v2.Task/Start is not supported",
        ))
    }

    async fn state(&self, r: StateRequest) -> Result<StateResponse> {
        log::info!("{r:#?}");
        Err(Status::not_found(
            "/containerd.task.v2.Task/State is not supported",
        ))
    }

    async fn stats(&self, r: StatsRequest) -> Result<StatsResponse> {
        log::info!("{r:#?}");
        Err(Status::not_found(
            "/containerd.task.v2.Task/Stats is not supported",
        ))
    }

    async fn update(&self, r: UpdateTaskRequest) -> Result<()> {
        log::info!("{r:#?}");
        Ok(())
    }

    async fn wait(&self, r: WaitRequest) -> Result<WaitResponse> {
        log::info!("{r:#?}");
        Err(Status::not_found(
            "/containerd.task.v2.Task/Wait is not supported",
        ))
    }

    async fn cleanup(&self, r: CleanupRequest) -> trapeze::Result<DeleteResponse> {
        log::info!("{r:#?}");
        Ok(DeleteResponse {
            exit_status: 137,
            exited_at: Some(SystemTime::now().into()),
            ..Default::default()
        })
    }

    async fn version(&self, _: ()) -> trapeze::Result<VersionResponse> {
        let executable = current_exe()?
            .file_stem()
            .context("Unknown executable")?
            .to_string_lossy()
            .into();

        Ok(VersionResponse {
            executable,
            info: vec![
                ("Runtime", "logger").into(),
                ("Version", "0.1.0").into(),
                ("Revision", "<none>").into(),
            ],
        })
    }
}
