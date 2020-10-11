#[cfg(feature = "web")]
use fixi_web::Config as WebConfig;
use {
    fixity::{Fixity, StorageWrite},
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
    #[structopt(long, default_value = "_storage")]
    pub storage_path: PathBuf,
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
    },
    // Fetch {},
}
#[derive(Debug, thiserror::Error)]
pub enum Error {
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
                let s = fixity::storage::fs::Fs::new(fixity::storage::fs::Config {
                    path: opt.fixi_opt.storage_path.clone(),
                })
                .await?;
                Fixity::new()
                    .with_storage(s)
                    .build()
                    .expect("constructing Fixity")
            };

            match cmd {
                RawCommand::Get { address } => cmd_raw_get(address).await,
                RawCommand::Put { with_input } => cmd_raw_put(fixi, with_input).await,
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
async fn cmd_raw_put<S>(fixi: Fixity<S>, with_input: Option<String>) -> Result<(), Error>
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
