//! [`SyncResponse`].

#[allow(unused_imports)]
use super::*;

#[derive(serde::Serialize)]
pub(crate) struct SyncResponse {
    pub(crate) synced: usize,
}
