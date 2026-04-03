use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use tabled::{Table, Tabled};

pub mod deploy_manager {
    tonic::include_proto!("deploy_manager");
}

pub mod managed_application {
    tonic::include_proto!("managed_application");
}

use deploy_manager::{DeployRequest, EnvVar, deploy_manager_client::DeployManagerClient};

use managed_application::{InfoRequest, managed_application_client::ManagedApplicationClient};

/// Output format for command results
#[derive(ValueEnum, Clone, Debug, Default)]
enum OutputFormat {
    Json,
    #[default]
    Table,
}

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
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
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

/// JSON-serialisable representation of a single environment variable
#[derive(Serialize, Deserialize, Debug)]
struct EnvVarJson {
    key: String,
    value: String,
}

/// JSON-serialisable shape of a DeployRequest payload
#[derive(Serialize, Deserialize, Debug)]
struct DeployRequestJson {
    yaml_content: String,
    #[serde(default)]
    env_vars: Vec<EnvVarJson>,
}

/// Table row used when rendering the deploy response
#[derive(Tabled)]
struct DeployResponseRow {
    #[tabled(rename = "Success")]
    success: bool,
    #[tabled(rename = "Report")]
    report: String,
}

/// Table row used when rendering the info response
#[derive(Tabled)]
struct ListeningAddressRow {
    #[tabled(rename = "Address")]
    address: String,
    #[tabled(rename = "Services")]
    services: String,
}

/// Parse a KEY=VALUE string into an `EnvVar` proto message.
/// Returns an error if the string does not contain '='.
fn parse_env_var(s: &str) -> Result<EnvVar, String> {
    let (key, value) = s
        .split_once('=')
        .ok_or_else(|| format!("env-var '{}' must be in KEY=VALUE format", s))?;
    Ok(EnvVar {
        key: key.to_string(),
        value: value.to_string(),
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Deploy {
            yaml_content,
            env_vars,
            json,
        } => {
            let mut client = DeployManagerClient::connect(cli.address).await?;

            let request = if let Some(json_str) = json {
                let payload: DeployRequestJson = serde_json::from_str(&json_str)?;
                DeployRequest {
                    yaml_content: payload.yaml_content,
                    env_vars: payload
                        .env_vars
                        .into_iter()
                        .map(|e| EnvVar {
                            key: e.key,
                            value: e.value,
                        })
                        .collect(),
                }
            } else {
                let yaml =
                    yaml_content.ok_or("--yaml-content is required when --json is not provided")?;
                let parsed_env_vars = env_vars
                    .iter()
                    .map(|s| parse_env_var(s))
                    .collect::<Result<Vec<_>, _>>()?;
                DeployRequest {
                    yaml_content: yaml,
                    env_vars: parsed_env_vars,
                }
            };

            let response = client.deploy(request).await?.into_inner();

            match cli.output {
                OutputFormat::Json => {
                    let json_out = serde_json::json!({
                        "success": response.success,
                        "report": response.report,
                    });
                    println!("{}", serde_json::to_string_pretty(&json_out)?);
                }
                OutputFormat::Table => {
                    let rows: Vec<DeployResponseRow> = response
                        .report
                        .iter()
                        .enumerate()
                        .map(|(i, line)| DeployResponseRow {
                            success: if i == 0 { response.success } else { false },
                            report: line.clone(),
                        })
                        .collect();

                    let rows = if rows.is_empty() {
                        vec![DeployResponseRow {
                            success: response.success,
                            report: String::new(),
                        }]
                    } else {
                        rows
                    };

                    println!("{}", Table::new(rows));
                }
            }
        }
        Commands::Info => {
            let mut client = ManagedApplicationClient::connect(cli.address).await?;
            let response = client.info(InfoRequest {}).await?.into_inner();

            match cli.output {
                OutputFormat::Json => {
                    let addresses: Vec<serde_json::Value> = response
                        .listening_addresses
                        .iter()
                        .map(|a| {
                            serde_json::json!({
                                "address": a.address,
                                "services": a.services,
                            })
                        })
                        .collect();
                    let json_out = serde_json::json!({
                        "app_name": response.app_name,
                        "listening_addresses": addresses,
                    });
                    println!("{}", serde_json::to_string_pretty(&json_out)?);
                }
                OutputFormat::Table => {
                    println!("Application: {}", response.app_name);
                    let rows: Vec<ListeningAddressRow> = response
                        .listening_addresses
                        .iter()
                        .map(|a| ListeningAddressRow {
                            address: a.address.clone(),
                            services: a.services.join(", "),
                        })
                        .collect();
                    println!("{}", Table::new(rows));
                }
            }
        }
    }

    Ok(())
}
