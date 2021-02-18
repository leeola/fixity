use {
    crate::{
        error::TypeError,
        path::{Path, SegmentResolve, SegmentUpdate},
        primitive::{commitlog::CommitLog, prolly::refimpl},
        storage::{StorageRead, StorageWrite},
        value::{Key, Value},
        workspace::{Guard, Status, Workspace},
        Addr, Error,
    },
    std::{collections::HashMap, fmt, mem},
};
pub struct File<'f, S, W> {
    storage: &'f S,
    workspace: &'f W,
    path: Path,
}
impl<'f, S, W> File<'f, S, W> {
    pub fn new(storage: &'f S, workspace: &'f W, path: Path) -> Self {
        Self {
            storage,
            workspace,
            path,
        }
    }
}
