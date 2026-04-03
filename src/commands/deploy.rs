use serde::{Deserialize, Serialize};
use tabled::{Table, Tabled};
use tabled::settings::Style;

use crate::output::OutputFormat;
use crate::proto::deploy_manager::{
    DeployRequest, EnvVar, deploy_manager_client::DeployManagerClient,
};

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

pub async fn handle(
    address: String,
    output: OutputFormat,
    yaml_content: Option<String>,
    env_vars: Vec<String>,
    json: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DeployManagerClient::connect(address).await?;

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

    match output {
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

            println!("{}", Table::new(rows).with(Style::blank()));
        }
    }

    Ok(())
}
