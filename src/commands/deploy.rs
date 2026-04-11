use serde::{Deserialize, Serialize};
use tabled::{Table, Tabled};
use tabled::settings::Style;

use crate::client;
use crate::output::OutputFormat;

/// A single environment variable represented as a key-value pair.
#[derive(Serialize, Deserialize, Debug)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
}

/// Request payload for the Deploy operation.
#[derive(Serialize, Deserialize, Debug)]
struct DeployRequest {
    yaml_content: String,
    #[serde(default)]
    env_vars: Vec<EnvVar>,
}

/// Response payload for the Deploy operation.
#[derive(Serialize, Deserialize, Debug)]
struct DeployResponse {
    success: bool,
    report: Vec<String>,
}

/// JSON-serialisable shape of a DeployRequest payload (for --json flag)
#[derive(Serialize, Deserialize, Debug)]
struct DeployRequestJson {
    yaml_content: String,
    #[serde(default)]
    env_vars: Vec<EnvVar>,
}

/// Table row used when rendering the deploy response
#[derive(Tabled)]
struct DeployResponseRow {
    #[tabled(rename = "Success")]
    success: bool,
    #[tabled(rename = "Report")]
    report: String,
}

/// Parse a KEY=VALUE string into an [`EnvVar`].
/// Returns `Err` with a descriptive message if '=' is missing.
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
    peer: String,
    output: OutputFormat,
    yaml_content: Option<String>,
    env_vars: Vec<String>,
    json: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = if let Some(json_str) = json {
        let payload: DeployRequestJson = serde_json::from_str(&json_str)?;
        DeployRequest {
            yaml_content: payload.yaml_content,
            env_vars: payload.env_vars,
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

    let response: DeployResponse =
        client::call(&peer, "DeployManager", "Deploy", &request).await?;

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
