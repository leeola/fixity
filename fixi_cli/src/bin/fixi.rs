use {
    fixity::{
        fixity::Builder,
        path::Path,
        value::{Key, Value},
        Config, Fixity,
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
    #[structopt(long, env = "FIXI_BASE_PATH")]
    pub base_path: Option<PathBuf>,
    #[structopt(long, env = "FIXI_WORKSPACE", default_value = "default")]
    pub workspace: String,
}
#[derive(Debug, StructOpt)]
enum Command {
    Init,
    /// A Map interface to Fixity data.
    ///
    /// Map is a primarily low level interface, enabling insight and mutation on the raw
    /// Key-Value format of Fixity.
    Map {
        /// The destination to write a `Value` to.
        #[structopt(short = "p", long = "path", parse(try_from_str = Path::from_cli_str), default_value = "")]
        path: Path,
        #[structopt(subcommand)]
        subcmd: MapSubcmd,
    },
    /// A raw bytes interface to Fixity, allowing reading and writing of arbitrary bytes at the
    /// provided `Path` and deduplicated via content defined chunking.
    Bytes {
        /// The destination to write bytes to.
        #[structopt(short = "p", long = "path", parse(try_from_str = Path::from_cli_str), default_value = "")]
        path: Path,
        #[structopt(subcommand)]
        subcmd: BytesSubcmd,
    },
}
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error: {0}")]
    User(String),
    #[error("fixity error: {0}")]
    Fixity(#[from] fixity::Error),
    #[error("fixity storage error: {0}")]
    StorageError(#[from] fixity::core::storage::Error),
}
#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("error")).init();
    let opt = Opt::from_args();

    let fixi = {
        let FixiOpt {
            fixi_dir_name,
            base_path,
            workspace,
        } = opt.fixi_opt;
        let config = Config::builder()
            .with_fixi_dir_name(fixi_dir_name)
            .with_base_path(base_path)
            .with_workspace_name(Some(workspace));
        let builder = Fixity::builder().with_config(config.clone());
        match opt.subcmd {
            Command::Init => {
                return cmd_init(builder).await;
            },
            _ => builder.open().await?,
        }
    };

    match opt.subcmd {
        Command::Init => unreachable!("matched above"),
        Command::Map { path, subcmd } => cmd_map(fixi, path, subcmd).await,
        Command::Bytes { path, subcmd } => cmd_bytes(fixi, path, subcmd).await,
    }
}
async fn cmd_init(builder: Builder) -> Result<(), Error> {
    let (_, config) = builder.init_config().await?;
    println!(
        "created Fixity repository at: {}",
        config.fixi_dir_path.to_string_lossy()
    );
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
        /// Optionally immediately commit this and any staged changes.
        #[structopt(short = "c", long = "commit")]
        commit: bool,
        /// The `Key` to write a `Value` or Bytes to.
        #[structopt(name = "KEY", parse(try_from_str = Key::from_cli_str))]
        key: Key,
        /// Write the [`Value`] to the given [`Key`].
        #[structopt(
             name = "VALUE", parse(try_from_str = Value::from_cli_str),
         )]
        value: Value,
    },
    Ls {
        #[structopt(short = "s", long = "start", parse(try_from_str = Key::from_cli_str))]
        start: Option<Key>,
        #[structopt(short = "e", long = "end", parse(try_from_str = Key::from_cli_str))]
        end: Option<Key>,
    },
}
async fn cmd_map(fixi: Fixity, path: Path, subcmd: MapSubcmd) -> Result<(), Error> {
    match subcmd {
        MapSubcmd::Get { key } => cmd_map_get(fixi, path, key).await,
        MapSubcmd::Put { commit, key, value } => cmd_map_put(fixi, path, commit, key, value).await,
        MapSubcmd::Ls { start, end } => cmd_map_ls(fixi, path, start, end).await,
    }
}
async fn cmd_map_get(fixi: Fixity, path: Path, key: Key) -> Result<(), Error> {
    let map = fixi.map(path);
    let v = map.get(key).await?;
    println!("{:?}", v);
    Ok(())
}
async fn cmd_map_put(
    fixi: Fixity,
    path: Path,
    commit: bool,
    key: Key,
    value: Value,
) -> Result<(), Error> {
    let mut map = fixi.map(path);
    map.insert(key, value).await?;
    if commit {
        let addr = map.commit().await?;
        println!("{:?}", addr);
    }
    Ok(())
}
async fn cmd_map_ls(
    fixi: Fixity,
    path: Path,
    start: Option<Key>,
    end: Option<Key>,
) -> Result<(), Error> {
    let map = fixi.map(path);
    let mut iter = match (start, end) {
        (Some(start), Some(end)) => map.iter(start..end).await?,
        (Some(start), None) => map.iter(start..).await?,
        (None, Some(end)) => map.iter(..end).await?,
        (None, None) => map.iter(..).await?,
    };
    while let Some(res) = iter.next() {
        let (key, value) = res?;
        println!("{}={}", key, value);
    }
    Ok(())
}
#[derive(Debug, StructOpt)]
enum BytesSubcmd {
    Get,
    Put {
        /// Optionally immediately commit this and any staged changes.
        #[structopt(short = "c", long = "commit")]
        commit: bool,
    },
}
async fn cmd_bytes(fixi: Fixity, path: Path, subcmd: BytesSubcmd) -> Result<(), Error> {
    match subcmd {
        BytesSubcmd::Get => cmd_bytes_get(fixi, path).await,
        BytesSubcmd::Put { commit } => cmd_bytes_put(fixi, path, commit).await,
    }
}
async fn cmd_bytes_get(fixi: Fixity, path: Path) -> Result<(), Error> {
    let stdout = tokio::io::stdout();
    let bytes = fixi.bytes(path)?;
    let _ = bytes.read(stdout).await?;
    Ok(())
}
async fn cmd_bytes_put(fixi: Fixity, path: Path, commit: bool) -> Result<(), Error> {
    if path.len() == 0 {
        return Err(Error::User(
            "cannot get/put bytes to root of fixity repository".to_owned(),
        ));
    }
    // TODO: tokio docs recommend against this for interactive uses[1], so this
    // should be fixed eventually - when interactivity is prioritized a bit more.
    //
    // Excerpt for context:
    //
    // > This handle is best used for non-interactive uses, such as when a file is piped
    // > into the application. For technical reasons, stdin is implemented by using an ordinary
    // > blocking read on a separate thread, and it is impossible to cancel that read.
    // > This can make shutdown of the runtime hang until the user presses enter.
    // >
    // > For interactive uses, it is recommended to spawn a thread dedicated to user input and
    // > use blocking IO directly in that thread.
    //
    // [1]: https://docs.rs/tokio/1.2.0/tokio/io/struct.Stdin.html
    let stdin = tokio::io::stdin();
    let bytes = fixi.bytes(path)?;
    let _ = bytes.write(stdin).await?;
    if commit {
        let addr = bytes.commit().await?;
        println!("{:?}", addr);
    } else {
        println!("bytes staged");
    }
    Ok(())
}
