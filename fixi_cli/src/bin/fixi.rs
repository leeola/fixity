#[cfg(feature = "web")]
use fixi_web::Config as WebConfig;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "fixi", about = "fixity content management")]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
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
    // Put {
    //     #[structopt(long)]
    //     stdin: bool,
    // },
    // Fetch {},
}
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("fixity error: {0}")]
    Fixity(String),
}
#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("error")).init();
    let opt = Opt::from_args();
    match opt.cmd {
        Command::Raw(cmd) => match cmd {
            RawCommand::Get { address } => cmd_raw_get(address).await,
        },
        #[cfg(feature = "web")]
        Command::Web(c) => unimplemented!("web serve"),
        // Command::Web(c) => fixi_web::serve(c).await,
    }
}
async fn cmd_raw_get(address: String) -> Result<(), Error> {
    Err(Error::Fixity("not implemented".into()))
}
