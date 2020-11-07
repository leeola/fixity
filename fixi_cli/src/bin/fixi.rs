#[cfg(feature = "web")]
use fixi_web::Config as WebConfig;
use {
    fixity::{value::Value, Fixity, Path, StorageWrite},
    std::path::PathBuf,
    structopt::StructOpt,
};

#[derive(Debug, StructOpt)]
#[structopt(name = "fixi", about = "fixity content management")]
struct Opt {
    #[structopt(flatten)]
    fixi_opt: FixiOpt,
    #[structopt(subcommand)]
    cmd: Command,
}
/// An temporary config setting up Fixi with the limited in-dev options
/// it has at the moment.
///
/// In the near future this will be revamped to support complex configuration,
/// which may or may not be managed by StructOpt.
#[derive(Debug, StructOpt)]
struct FixiOpt {
    #[structopt(long, env = "FIXI_DIR")]
    pub fixi_dir: PathBuf,
    #[structopt(long, env = "FIXI_WORKSPACE", default_value = "default")]
    pub workspace: String,
    #[structopt(long, env = "FIXI_STORAGE_PATH")]
    pub storage_path: Option<PathBuf>,
}
#[derive(Debug, StructOpt)]
enum Command {
    Raw(RawCommand),
    #[cfg(feature = "web")]
    Web(WebConfig),
}
#[derive(Debug, StructOpt)]
enum RawCommand {
    Get {
        /// The fixity address to get.
        address: String,
    },
    Put {
        /// Put with the provided String instead of using Stdin.
        #[structopt(long, short = "i")]
        with_input: Option<String>,
        /// Construct a Fixity Path from the provided keys.
        #[structopt(name = "PATH", parse(try_from_str = Path::from_cli_str))]
        path: Path,
        #[structopt(name = "VALUE", parse(try_from_str = Value::from_cli_str))]
        value: Option<Value>,
    },
    // Fetch {},
}
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error: {0}")]
    User(String),
    #[error("fixity error: {0}")]
    Fixity(#[from] fixity::Error),
    #[error("fixity storage error: {0}")]
    StorageError(#[from] fixity::storage::Error),
}
#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("error")).init();
    let opt = Opt::from_args();
    match opt.cmd {
        Command::Raw(cmd) => {
            let fixi = {
                let FixiOpt {
                    storage_path,
                    fixi_dir,
                    workspace,
                } = opt.fixi_opt;
                let storage_path = storage_path.unwrap_or_else(|| fixi_dir.join("storage"));
                let s = fixity::storage::fs::Fs::new(fixity::storage::fs::Config {
                    path: storage_path,
                })
                .await?;
                Fixity::new()
                    .with_fixity_dir(fixi_dir)
                    .with_storage(s)
                    .with_workspace(workspace)
                    .build()
                    .expect("constructing Fixity")
            };

            match cmd {
                RawCommand::Get { address } => cmd_raw_get(address).await,
                RawCommand::Put {
                    with_input,
                    path,
                    value,
                } => cmd_raw_put(fixi, with_input, path, value).await,
            }
        }
        #[cfg(feature = "web")]
        Command::Web(c) => unimplemented!("web serve"),
        // Command::Web(c) => fixi_web::serve(c).await,
    }
}
async fn cmd_raw_get(_address: String) -> Result<(), Error> {
    unimplemented!("cmd_raw_get")
}
async fn cmd_raw_put<S>(
    fixi: Fixity<S>,
    with_input: Option<String>,
    path: Path,
    value: Option<Value>,
) -> Result<(), Error>
where
    S: StorageWrite,
{
    let addr = if let Some(s) = with_input {
        fixi.put_reader(s.as_bytes()).await?
    } else {
        fixi.put_reader(tokio::io::stdin()).await?
    };
    println!("{}", addr);
    Ok(())
}
