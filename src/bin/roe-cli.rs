use clap::Parser;
use roe::{commands, output::OutputFormat};

#[derive(Parser, Debug)]
#[command(
    name = "roe-cli",
    about = "CLI client for the roe gRPC services",
    version
)]
struct Cli {
    /// gRPC server address (e.g. http://[::1]:50051)
    #[arg(short, long, default_value = "http://[::1]:50051")]
    address: String,

    /// Output format: json or table
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Table)]
    output: OutputFormat,

    #[command(subcommand)]
    command: commands::Commands,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    commands::run(cli.command, cli.address, cli.output).await
}
