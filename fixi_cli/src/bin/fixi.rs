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
    Get {},
    Put {
        #[structopt(long)]
        stdin: bool,
    },
    Fetch {},
}

#[tokio::main]
async fn main() -> () {
    let opt = Opt::from_args();
    match opt.cmd {
        Command::Raw(_) => unimplemented!("raw cmd"),
        #[cfg(feature = "web")]
        Command::Web(c) => fixi_web::serve(c).await,
    }
}
