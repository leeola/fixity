//!
//! A thin wrapper around [`git_fixi::cli::main`].
#[tokio::main]
async fn main() -> Result<(), String> {
    git_fixi::cli::main().await
}
