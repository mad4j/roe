pub mod deploy;
pub mod info;

use clap::Subcommand;

use crate::output::OutputFormat;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Call the Deploy RPC on the DeployManager service
    Deploy {
        /// YAML configuration content (required unless --json is provided)
        #[arg(long)]
        yaml_content: Option<String>,

        /// Environment variable in KEY=VALUE format (repeatable)
        #[arg(long = "env-var")]
        env_vars: Vec<String>,

        /// Provide all parameters as a JSON object instead of individual flags.
        /// Example: '{"yaml_content":"name: app","env_vars":[{"key":"ENV","value":"prod"}]}'
        #[arg(long, conflicts_with_all = ["yaml_content", "env_vars"])]
        json: Option<String>,
    },
    /// Call the Info RPC on the ManagedApplication service
    Info,
}

pub async fn run(
    command: Commands,
    address: String,
    output: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Deploy {
            yaml_content,
            env_vars,
            json,
        } => deploy::handle(address, output, yaml_content, env_vars, json).await,
        Commands::Info => info::handle(address, output).await,
    }
}
