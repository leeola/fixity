use {std::path::PathBuf, structopt::StructOpt};
/// An temporary config setting up Fixi with the limited in-dev options
/// it has at the moment.
///
/// In the near future this will be revamped to support complex configuration,
/// which may or may not be managed by StructOpt.
#[derive(Debug, StructOpt)]
struct FixiOpt {
    // #[structopt(long, env = "GLOBAL_FIXI_DIR")]
    // pub global_fixi_dir: Option<PathBuf>,
    #[structopt(long, env = "FIXI_DIR_NAME")]
    pub fixi_dir_name: Option<PathBuf>,
    #[structopt(long, env = "FIXI_DIR")]
    pub fixi_dir: Option<PathBuf>,
    #[structopt(long, env = "FIXI_WORKSPACE", default_value = "default")]
    pub workspace: String,
    #[structopt(long, env = "FIXI_STORAGE_DIR")]
    pub storage_dir: Option<PathBuf>,
}
#[derive(Debug, StructOpt)]
#[structopt(name = "fixi", about = "fixity content management")]
struct Opt {
    #[structopt(flatten)]
    fixi_opt: FixiOpt,
    #[structopt(subcommand)]
    subcmd: Command,
}
#[derive(Debug, StructOpt)]
enum Command {
    Clean,
    Smudge,
}
/*
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error: {0}")]
    User(String),
    #[error("fixity error: {0}")]
    Fixity(#[from] fixity::Error),
    #[error("fixity storage error: {0}")]
    StorageError(#[from] fixity::storage::Error),
}
*/
#[tokio::main]
async fn main() -> Result<(), String> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("error")).init();
    let opt = Opt::from_args();

    /*
    let fixi = {
        let FixiOpt {
            fixi_dir_name,
            fixi_dir,
            workspace,
            storage_dir,
        } = opt.fixi_opt;
        fixity::Fixity::<storage::Fs, workspace::Fs>::builder()
            .fixi_dir_name(fixi_dir_name)
            .fixi_dir(fixi_dir)
            .with_workspace_name(workspace)
            .fs_storage_dir(storage_dir)
            .open()
            .await?
    };
    */

    println!("stdout print");
    eprintln!("stderr print");
    Ok(())
}
