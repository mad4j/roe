pub mod application;
pub mod deploy;
pub mod info;
pub mod terminate;

use clap::Subcommand;

use crate::output::OutputFormat;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Call RPCs on the ApplicationManager service
    Application {
        #[command(subcommand)]
        command: ApplicationCommands,
    },
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
    /// Call the Terminate RPC on the ManagedApplication service
    Terminate {
        /// Optional human-readable reason for the termination request
        #[arg(long)]
        reason: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ApplicationCommands {
    /// Call ActivateApplication on the ApplicationManager service
    Activate {
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
    /// Call ListActiveApplications on the ApplicationManager service
    List,
    /// Call TerminateApplication on the ApplicationManager service
    Terminate {
        /// Active application identifier (required unless --json is provided)
        #[arg(long)]
        application_id: Option<String>,

        /// Optional human-readable reason for the termination request
        #[arg(long)]
        reason: Option<String>,

        /// Provide all parameters as a JSON object instead of individual flags.
        /// Example: '{"application_id":"app-123","reason":"maintenance"}'
        #[arg(long, conflicts_with_all = ["application_id", "reason"])]
        json: Option<String>,
    },
}

pub async fn run(
    command: Commands,
    address: String,
    output: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Application { command } => match command {
            ApplicationCommands::Activate {
                yaml_content,
                env_vars,
                json,
            } => application::activate(address, output, yaml_content, env_vars, json).await,
            ApplicationCommands::List => application::list(address, output).await,
            ApplicationCommands::Terminate {
                application_id,
                reason,
                json,
            } => {
                application::terminate(address, output, application_id, reason, json).await
            }
        },
        Commands::Deploy {
            yaml_content,
            env_vars,
            json,
        } => deploy::handle(address, output, yaml_content, env_vars, json).await,
        Commands::Info => info::handle(address, output).await,
        Commands::Terminate { reason } => terminate::handle(address, output, reason).await,
    }
}
