mod cli;

#[tokio::main]
async fn main() {
    std::process::exit(cli::run().await);
}
