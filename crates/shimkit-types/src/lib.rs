#[doc(hidden)]
pub mod protos {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/mod.rs"));
}

pub mod events {
    pub use super::protos::containerd::events::*;
    pub use super::protos::containerd::services::events::ttrpc::v1::*;
}

pub mod task {
    pub use super::protos::containerd::task::v2::*;
    pub use super::protos::containerd::types::*;
    pub use super::protos::containerd::v1::types::*;
}

pub mod sandbox {
    pub use super::protos::containerd::runtime::sandbox::v1::*;
    pub use super::protos::containerd::types::*;
}

pub mod cri {
    pub use super::protos::runtime::v1::*;
}

pub use prost_types as prost;
pub use trapeze::{Code, Result, Status};

impl<K: ToString, V: ToString> From<&(K, V)> for task::KeyValue {
    fn from(value: &(K, V)) -> Self {
        Self {
            key: value.0.to_string(),
            value: value.1.to_string(),
        }
    }
}

impl<K: ToString, V: ToString> From<(K, V)> for task::KeyValue {
    fn from(value: (K, V)) -> Self {
        Self {
            key: value.0.to_string(),
            value: value.1.to_string(),
        }
    }
}
