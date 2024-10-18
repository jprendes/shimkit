use shimkit::protos::containerd::task::v2::*;
use trapeze::{Result, Status};

use super::Server;

impl Task for Server {
    async fn checkpoint(&self, r: CheckpointTaskRequest) -> Result<()> {
        println!("{r:#?}");
        Ok(())
    }

    async fn close_io(&self, r: CloseIoRequest) -> Result<()> {
        println!("{r:#?}");
        Ok(())
    }
    async fn connect(&self, r: ConnectRequest) -> Result<ConnectResponse> {
        println!("{r:#?}");
        Err(Status::not_found(
            "/containerd.task.v2.Task/Connect is not supported",
        ))
    }
    async fn create(&self, r: CreateTaskRequest) -> Result<CreateTaskResponse> {
        println!("{r:#?}");
        Err(Status::not_found(
            "/containerd.task.v2.Task/Create is not supported",
        ))
    }
    async fn delete(&self, r: DeleteRequest) -> Result<DeleteResponse> {
        println!("{r:#?}");
        Err(Status::not_found(
            "/containerd.task.v2.Task/Delete is not supported",
        ))
    }
    async fn exec(&self, r: ExecProcessRequest) -> Result<()> {
        println!("{r:#?}");
        Ok(())
    }
    async fn kill(&self, r: KillRequest) -> Result<()> {
        println!("{r:#?}");
        Ok(())
    }
    async fn pause(&self, r: PauseRequest) -> Result<()> {
        println!("{r:#?}");
        Ok(())
    }
    async fn pids(&self, r: PidsRequest) -> Result<PidsResponse> {
        println!("{r:#?}");
        Err(Status::not_found(
            "/containerd.task.v2.Task/Pids is not supported",
        ))
    }
    async fn resize_pty(&self, r: ResizePtyRequest) -> Result<()> {
        println!("{r:#?}");
        Ok(())
    }
    async fn resume(&self, r: ResumeRequest) -> Result<()> {
        println!("{r:#?}");
        Ok(())
    }
    async fn shutdown(&self, r: ShutdownRequest) -> Result<()> {
        println!("{r:#?}");
        Ok(())
    }
    async fn start(&self, r: StartRequest) -> Result<StartResponse> {
        println!("{r:#?}");
        Err(Status::not_found(
            "/containerd.task.v2.Task/Start is not supported",
        ))
    }
    async fn state(&self, r: StateRequest) -> Result<StateResponse> {
        println!("{r:#?}");
        Err(Status::not_found(
            "/containerd.task.v2.Task/State is not supported",
        ))
    }
    async fn stats(&self, r: StatsRequest) -> Result<StatsResponse> {
        println!("{r:#?}");
        Err(Status::not_found(
            "/containerd.task.v2.Task/Stats is not supported",
        ))
    }
    async fn update(&self, r: UpdateTaskRequest) -> Result<()> {
        println!("{r:#?}");
        Ok(())
    }
    async fn wait(&self, r: WaitRequest) -> Result<WaitResponse> {
        println!("{r:#?}");
        Err(Status::not_found(
            "/containerd.task.v2.Task/Wait is not supported",
        ))
    }
}