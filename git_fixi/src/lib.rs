pub mod cli {
    use {
        fixity::{
            path::{MapSegment, Path},
            Fixity,
        },
        std::path::PathBuf,
        structopt::StructOpt,
    };
    /// An temporary config setting up Fixi with the limited in-dev options
    /// it has at the moment.
    ///
    /// In the near future this will be revamped to support complex configuration,
    /// which may or may not be managed by StructOpt.
    #[derive(Debug, StructOpt)]
    pub struct FixiOpt {
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
    #[structopt(name = "fixi", about = "fixity content management")]
    pub struct Opt {
        #[structopt(flatten)]
        fixi_opt: FixiOpt,
        #[structopt(subcommand)]
        subcmd: Command,
    }
    #[derive(Debug, StructOpt)]
    pub enum Command {
        Clean { file_name: String },
        Smudge { file_name: String },
    }
    pub async fn main() -> Result<(), String> {
        env_logger::from_env(env_logger::Env::default().default_filter_or("error")).init();

        log::info!("woo");

        let opt = Opt::from_args();
        dbg!(&opt);
        let fixi = {
            let FixiOpt {
                fixi_dir_name,
                base_path,
                workspace,
            } = opt.fixi_opt;
            let config = fixity::Config::builder()
                .with_fixi_dir_name(fixi_dir_name)
                .with_base_path(base_path)
                .with_workspace_name(Some(workspace));
            Fixity::builder()
                .with_config(config)
                .open()
                .await
                .map_err(|err| format!("{}", err))?
        };
        match opt.subcmd {
            Command::Clean { file_name } => cmd_clean(fixi, file_name).await,
            Command::Smudge { file_name } => cmd_smudge(fixi, file_name).await,
        }
    }
    pub async fn cmd_clean(fixi: Fixity, _file_name: String) -> Result<(), String> {
        let path = Path::new().into_push(MapSegment::from("file_name"));

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
        let bytes = fixi.bytes(path).map_err(|err| format!("{}", err))?;
        // TODO: probably should auto commit, but the read needs to be a static reference
        // to the content, not commit.
        // Some new Fixity APIs are needed here.
        let addr = bytes.write(stdin).await.map_err(|err| format!("{}", err))?;
        println!("{:?}", addr);
        eprintln!("stderr print, {:?}", addr);
        Ok(())
    }
    pub async fn cmd_smudge(_fixi: Fixity, _file_name: String) -> Result<(), String> {
        todo!()
    }
}
