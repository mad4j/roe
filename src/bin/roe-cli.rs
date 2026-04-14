use clap::Parser;
use roe::{commands, output::OutputFormat};

#[derive(Parser, Debug)]
#[command(
    name = "roe-cli",
    about = "CLI client for the roe HDDS services",
    version
)]
struct Cli {
    /// DDS peer address (e.g. 127.0.0.1:7411).
    /// Leave empty to rely on RTPS multicast discovery.
    #[arg(short, long, default_value = "127.0.0.1:7411")]
    peer: String,

    /// Output format: json or table
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Table)]
    output: OutputFormat,

    #[command(subcommand)]
    command: commands::Commands,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    commands::run(cli.command, cli.peer, cli.output).await
}
