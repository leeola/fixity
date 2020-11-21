use std::{
    io,
    path::{Path, PathBuf},
};

/// Resolve an existing fixity dir above the current directory.
fn resolve<P>(fixi_dir_name: P, mut root: PathBuf) -> Option<PathBuf>
where
    P: AsRef<Path>,
{
    let fixi_dir = root.join(&fixi_dir_name);
    if fixi_dir.exists() {
        return Some(fixi_dir);
    }
    while root.pop() {
        let fixi_dir = root.join(&fixi_dir_name);
        if fixi_dir.exists() {
            return Some(fixi_dir);
        }
    }
    None
}
