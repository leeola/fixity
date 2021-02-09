#[cfg(feature = "web")]
use fixi_web::Config as WebConfig;
use {
    fixity::{
        fixity::Builder,
        path::Path,
        storage,
        value::{Key, Value},
        workspace::{self, Workspace},
        Fixity, Storage,
    },
    std::path::PathBuf,
    structopt::StructOpt,
};

#[derive(Debug, StructOpt)]
#[structopt(name = "fixi", about = "fixity content management")]
struct Opt {
    #[structopt(flatten)]
    fixi_opt: FixiOpt,
    #[structopt(subcommand)]
    subcmd: Command,
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
enum Command {
    Init,
    /// A Map interface to Fixity data.
    ///
    /// Map is a primarily low level interface, enabling insight and mutation on the raw
    /// Key-Value format of Fixity.
    Map {
        /// The destination to write a `Value` or Bytes to.
        #[structopt(short = "p", long = "path", parse(try_from_str = Path::from_cli_str), default_value = "")]
        path: Path,
        #[structopt(subcommand)]
        subcmd: MapSubcmd,
    },
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

    let fixi = {
        let FixiOpt {
            fixi_dir_name,
            fixi_dir,
            workspace,
            storage_dir,
        } = opt.fixi_opt;
        let builder = fixity::Fixity::<storage::Fs, workspace::Fs>::builder()
            .fixi_dir_name(fixi_dir_name)
            .fixi_dir(fixi_dir)
            .with_workspace_name(workspace)
            .fs_storage_dir(storage_dir);
        match opt.subcmd {
            Command::Init => return cmd_init(builder).await,
            _ => builder.open().await?,
        }
    };

    match opt.subcmd {
        Command::Init => unreachable!("matched above"),
        Command::Map { path, subcmd } => cmd_map(fixi, path, subcmd).await,
        #[cfg(feature = "web")]
        Command::Web(c) => unimplemented!("web serve"),
        // Command::Web(c) => fixi_web::serve(c).await,
    }
}
async fn cmd_init(b: Builder<storage::Fs, workspace::Fs>) -> Result<(), Error> {
    b.init().await?;
    Ok(())
}
#[derive(Debug, StructOpt)]
enum MapSubcmd {
    Get {
        /// The `Key` to get a `Value` from.
        #[structopt(name = "KEY", parse(try_from_str = Key::from_cli_str))]
        key: Key,
    },
    Put {
        /// Write stdin to the given [`Path`].
        #[structopt(long, short = "i")]
        stdin: bool,
        /// The `Key` to write a `Value` or Bytes to.
        #[structopt(name = "KEY", parse(try_from_str = Key::from_cli_str))]
        key: Key,
        /// Write the [`Value`] to the given [`Key`].
        #[structopt(
             name = "VALUE", parse(try_from_str = Value::from_cli_str),
             required_unless("stdin"),
         )]
        value: Option<Value>,
    },
}
async fn cmd_map<S, W>(fixi: Fixity<S, W>, path: Path, subcmd: MapSubcmd) -> Result<(), Error>
where
    S: Storage,
    W: Workspace,
{
    match subcmd {
        MapSubcmd::Get { key } => cmd_get(fixi, path, key).await,
        MapSubcmd::Put { stdin, key, value } => match (stdin, value) {
            (false, Some(value)) => cmd_put_value(fixi, path, key, value).await,
            (true, None) => cmd_put_stdin(fixi, todo!()).await,
            _ => unreachable!("Structopt should be configured to make this unreachable"),
        },
    }
}
async fn cmd_get<S, W>(fixi: Fixity<S, W>, path: Path, key: Key) -> Result<(), Error>
where
    S: Storage,
    W: Workspace,
{
    let map = fixi.map(path);
    let v = map.get(key).await?;
    println!("{:?}", v);
    Ok(())
}
async fn cmd_put_stdin<S, W>(fixi: Fixity<S, W>, _path: Path) -> Result<(), Error>
where
    S: Storage,
    W: Workspace,
{
    let addr = fixi.put_reader(tokio::io::stdin()).await?;
    println!("{:?}", addr);
    Ok(())
}
async fn cmd_put_value<S, W>(
    fixi: Fixity<S, W>,
    path: Path,
    key: Key,
    value: Value,
) -> Result<(), Error>
where
    S: Storage,
    W: Workspace,
{
    let mut map = fixi.map(path);
    map.insert(key, value);
    map.stage().await?;
    let addr = map.commit().await?;
    println!("{:?}", addr);
    Ok(())
}
