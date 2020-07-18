use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "fixi", about = "fixity content management")]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    Get {},
    Put {
        #[structopt(long)]
        stdin: bool,
    },
    Fetch {},
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
