//! [`SyncResponse`].

#[derive(serde::Serialize)]
pub(crate) struct SyncResponse {
    pub(crate) synced: usize,
}
