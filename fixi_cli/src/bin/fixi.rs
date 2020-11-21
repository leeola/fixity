#[cfg(feature = "web")]
use fixi_web::Config as WebConfig;
use {
    fixity::{storage::Fs, value::Value, Fixity, Path, Storage},
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
    // #[structopt(long, env = "GLOBAL_FIXI_DIR")]
    // pub global_fixi_dir: Option<PathBuf>,
    #[structopt(long, env = "FIXI_DIR_NAME", default_value = ".fixi")]
    pub fixi_dir_name: PathBuf,
    #[structopt(long, env = "FIXI_PATH")]
    pub fixi_path: Option<PathBuf>,
    #[structopt(long, env = "FIXI_WORKSPACE", default_value = "default")]
    pub workspace: String,
    #[structopt(long, env = "FIXI_STORAGE_DIR")]
    pub storage_dir: Option<PathBuf>,
}
#[derive(Debug, StructOpt)]
enum Command {
    Init,
    Get {
        /// The Path to get a `Value` from.
        #[structopt(name = "PATH", parse(try_from_str = Path::from_cli_str))]
        path: Path,
    },
    Put {
        /// Write stdin to the given [`Path`].
        #[structopt(long, short = "i")]
        stdin: bool,
        /// The destination to write a `Value` or Bytes to.
        #[structopt(name = "PATH", parse(try_from_str = Path::from_cli_str))]
        path: Path,
        /// Write the [`Value`] to the given [`Path`].
        #[structopt(
            name = "VALUE", parse(try_from_str = Value::from_cli_str),
            required_unless("stdin"),
        )]
        value: Option<Value>,
    },
    // Raw(RawCommand),
    #[cfg(feature = "web")]
    Web(WebConfig),
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

    let FixiOpt {
        fixi_dir_name,
        fixi_path,
        workspace,
        storage_dir,
    } = opt.fixi_opt;
    let fixi_dir = match (fixi_dir_name, fixi_path) {
        (_, Some(fixi_path)) => fixi_path,
        (fixi_dir_name, None) => fixi_dir_name,
    };
    let storage_dir = storage_dir.unwrap_or_else(|| fixi_dir.join("storage"));

    match opt.cmd {
        Command::Init => return cmd_init(fixi_dir, workspace, storage_dir).await,
        _ => {}
    }

    let fixi = {
        fixity::Fixity::<Fs>::open(
            fixi_dir,
            workspace,
            fixity::storage::fs::Config { path: storage_dir },
        )
        .await?
    };

    match opt.cmd {
        Command::Init => unreachable!("matched above"),
        Command::Get { path } => cmd_get(fixi, path).await,
        Command::Put { stdin, path, value } => match (stdin, value) {
            (false, Some(value)) => cmd_put_value(fixi, path, value).await,
            (true, None) => cmd_put_stdin(fixi, path).await,
            _ => unreachable!("Structopt should be configured to make this unreachable"),
        },
        #[cfg(feature = "web")]
        Command::Web(c) => unimplemented!("web serve"),
        // Command::Web(c) => fixi_web::serve(c).await,
    }
}
async fn cmd_init(fixi_dir: PathBuf, workspace: String, storage_dir: PathBuf) -> Result<(), Error> {
    let _ = fixity::Fixity::<Fs>::init(
        fixi_dir,
        workspace,
        fixity::storage::fs::Config { path: storage_dir },
    )
    .await?;
    Ok(())
}
async fn cmd_get<S>(fixi: Fixity<S>, mut path: Path) -> Result<(), Error>
where
    S: Storage,
{
    let key = path.pop().expect("CLI interface enforces at least one key");
    let mut map = fixi.map(path).await?;
    let v = map.get(key).await?;
    dbg!(v);
    Ok(())
}
async fn cmd_put_stdin<S>(fixi: Fixity<S>, _path: Path) -> Result<(), Error>
where
    S: Storage,
{
    let addr = fixi.put_reader(tokio::io::stdin()).await?;
    println!("{}", addr);
    Ok(())
}
async fn cmd_put_value<S>(fixi: Fixity<S>, mut path: Path, value: Value) -> Result<(), Error>
where
    S: Storage,
{
    let key = path.pop().expect("CLI interface enforces at least one key");
    let mut map = fixi.map(path).await?;
    map.insert(key, value);
    let addr = map.commit().await?;
    dbg!(addr);
    Ok(())
}
