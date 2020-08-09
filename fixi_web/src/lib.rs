//! A simple local web server for inspecting Fixity data with a UI.
//
//! Eventually this is likely to be used with [Tauri](https://tauri.studio/en/).
#![feature(proc_macro_hygiene, decl_macro)]

use rocket::{get, routes};

/// The local web server config.
#[derive(Debug)]
#[cfg_attr(feature = "structopt", derive(structopt::StructOpt))]
pub struct Config {
    #[cfg_attr(feature = "structopt", structopt(default_value = "42"))]
    pub port: u32,
    #[cfg_attr(feature = "structopt", structopt(default_value = "localhost"))]
    pub host: String,
}
#[get("/")]
async fn index() -> String {
    "Hello, world!".to_string()
}
/// Serve the local fixi_web server.
pub async fn serve(_config: Config) {
    rocket::ignite()
        .mount("/", routes![index])
        .launch()
        .await
        .expect("launch failed");
}
