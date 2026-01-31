mod cli;
mod connection;
mod types;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    std::process::exit(cli::run().await);
}
