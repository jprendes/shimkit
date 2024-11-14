use std::io::Result as IoResult;
use std::sync::Arc;
use std::time::SystemTime;

use async_trait::async_trait;
use prost::Name;
use prost_types::{Any, Timestamp};
use trapeze::{Client, Result};

use crate::types::events::*;

// Types that can be encoded as `Any` implement the `Name` trait
pub trait Event: Name {
    fn topic(&self) -> &'static str;
}

struct NullEvents;
impl Events for NullEvents {
    async fn forward(&self, _: ForwardRequest) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
trait DynEvents {
    async fn forward(&self, forward_request: ForwardRequest) -> Result<()>;
}

#[async_trait]
impl<E: Events> DynEvents for E {
    async fn forward(&self, forward_request: ForwardRequest) -> Result<()> {
        Events::forward(self, forward_request).await
    }
}

/// Event publisher connects to containerd's TTRPC endpoint to publish events from shim.
#[derive(Clone)]
pub struct EventPublisher {
    events: Arc<dyn DynEvents + Send + Sync>,
    namespace: String,
}

impl EventPublisher {
    /// Connect to containerd's TTRPC endpoint
    pub(crate) async fn connect(address: impl AsRef<str>) -> IoResult<Self> {
        let client = Client::connect(address).await?;
        Ok(Self::new(client))
    }

    pub(crate) fn null() -> Self {
        Self::new(NullEvents)
    }

    pub(crate) fn new(events: impl Events) -> Self {
        Self {
            events: Arc::new(events),
            namespace: "".into(),
        }
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

        self.events.forward(req).await?;

        Ok(())
    }

    pub fn with_namespace(&self, namespace: impl Into<String>) -> Self {
        let mut this = self.clone();
        this.namespace = namespace.into();
        this
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

    use super::*;

    struct FakePublisher {
        tx: Sender<Envelope>,
    }

    impl Events for FakePublisher {
        async fn forward(&self, req: ForwardRequest) -> trapeze::Result<()> {
            let env = req.envelope.unwrap_or_default();
            self.tx.send(env).await.unwrap();
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_publish() {
        let (tx, mut rx) = channel(1);
        let server = FakePublisher { tx };

        let publisher = EventPublisher::new(server).with_namespace("ns1");
        let msg = TaskOom {
            container_id: "test".into(),
        };
        publisher.publish(msg).await.unwrap();

        let Envelope {
            topic,
            event,
            namespace,
            ..
        } = rx.recv().await.unwrap();
        assert_eq!(topic, "/tasks/oom");
        assert_eq!(namespace, "ns1");
        assert_eq!(
            event.unwrap().to_msg(),
            Ok(TaskOom {
                container_id: "test".into(),
            })
        );
    }
}
