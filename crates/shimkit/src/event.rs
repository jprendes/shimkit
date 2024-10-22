use std::io::Result as IoResult;
use std::time::SystemTime;

use prost::Name;
use prost_types::{Any, Timestamp};
use trapeze::transport::Connection;
use trapeze::{Client, Result};

use crate::protos::containerd::events::*;
use crate::protos::containerd::services::events::ttrpc::v1::{Envelope, Events, ForwardRequest};

// Types that can be encoded as `Any` implement the `Name` trait
pub trait Event: Name {
    fn topic(&self) -> &'static str;
}

/// Event publisher connects to containerd's TTRPC endpoint to publish events from shim.
#[derive(Clone)]
pub struct EventPublisher {
    client: Client,
    namespace: String,
}

impl EventPublisher {
    /// Connect to containerd's TTRPC endpoint
    pub async fn connect(address: impl AsRef<str>, namespace: impl Into<String>) -> IoResult<Self> {
        let client = Client::connect(address).await?;
        let namespace = namespace.into();
        Ok(Self { client, namespace })
    }

    pub fn new(connection: impl Connection, namespace: impl Into<String>) -> Self {
        let client = Client::new(connection);
        let namespace = namespace.into();
        Self { client, namespace }
    }

    /// Publish a new event.
    pub async fn publish(&self, event: impl Event) -> Result<()> {
        let req = ForwardRequest {
            envelope: Envelope {
                topic: event.topic().into(),
                timestamp: Timestamp::from(SystemTime::now()).into(),
                namespace: self.namespace.clone(),
                event: Any::from_msg(&event).unwrap().into(),
            }
            .into(),
        };

        self.client.forward(req).await?;

        Ok(())
    }
}

impl Event for TaskCreate {
    fn topic(&self) -> &'static str {
        "/tasks/create"
    }
}

impl Event for TaskStart {
    fn topic(&self) -> &'static str {
        "/tasks/start"
    }
}

impl Event for TaskExecAdded {
    fn topic(&self) -> &'static str {
        "/tasks/exec-added"
    }
}

impl Event for TaskExecStarted {
    fn topic(&self) -> &'static str {
        "/tasks/exec-started"
    }
}

impl Event for TaskPaused {
    fn topic(&self) -> &'static str {
        "/tasks/paused"
    }
}

impl Event for TaskResumed {
    fn topic(&self) -> &'static str {
        "/tasks/resumed"
    }
}

impl Event for TaskExit {
    fn topic(&self) -> &'static str {
        "/tasks/exit"
    }
}

impl Event for TaskDelete {
    fn topic(&self) -> &'static str {
        "/tasks/delete"
    }
}

impl Event for TaskOom {
    fn topic(&self) -> &'static str {
        "/tasks/oom"
    }
}

impl Event for TaskCheckpointed {
    fn topic(&self) -> &'static str {
        "/tasks/checkpointed"
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc::{channel, Sender};
    use trapeze::{service, ServerConnection};

    use super::*;

    struct FakeServer {
        tx: Sender<Envelope>,
    }

    impl Events for FakeServer {
        async fn forward(&self, req: ForwardRequest) -> trapeze::Result<()> {
            let env = req.envelope.unwrap_or_default();
            self.tx.send(env).await.unwrap();
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_publish() {
        let (client, server) = tokio::io::duplex(1024);

        let (tx, mut rx) = channel(1);
        tokio::spawn(async move {
            ServerConnection::new(server)
                .register(service!(FakeServer { tx } : Events))
                .start()
                .await
        });

        let publisher = EventPublisher::new(client, "ns1");
        let msg = TaskOom {
            container_id: "test".into(),
        };
        publisher.publish(msg).await.unwrap();

        let Envelope { topic, event, .. } = rx.recv().await.unwrap();
        assert_eq!(topic, "/tasks/oom");
        assert_eq!(
            event.unwrap().to_msg(),
            Ok(TaskOom {
                container_id: "test".into(),
            })
        );
    }
}
