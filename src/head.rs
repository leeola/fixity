use {crate::Addr, std::path::Path};
pub struct Head {}
impl Head {
    pub async fn open(_fixi_dir: &Path, _workspace: &str) -> Result<Self, Error> {
        todo!("open head")
    }
    pub fn addr(&self) -> Option<Addr> {
        todo!("head addr")
    }
}
#[derive(Debug, thiserror::Error)]
pub enum Error {}
