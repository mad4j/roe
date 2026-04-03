use serde::{Deserialize, Serialize};
use tabled::settings::Style;
use tabled::{Table, Tabled};

use crate::output::OutputFormat;
use crate::proto::application_manager::{
    ActivateApplicationRequest, ListActiveApplicationsRequest, TerminateApplicationRequest,
    application_manager_client::ApplicationManagerClient,
};
use crate::proto::deploy_manager::EnvVar;

/// JSON-serialisable representation of a single environment variable.
#[derive(Serialize, Deserialize, Debug)]
struct EnvVarJson {
    key: String,
    value: String,
}

/// JSON-serialisable shape of an ActivateApplication request payload.
#[derive(Serialize, Deserialize, Debug)]
struct ActivateApplicationRequestJson {
    yaml_content: String,
    #[serde(default)]
    env_vars: Vec<EnvVarJson>,
}

/// JSON-serialisable shape of a TerminateApplication request payload.
#[derive(Serialize, Deserialize, Debug)]
struct TerminateApplicationRequestJson {
    application_id: String,
    #[serde(default)]
    reason: String,
}

#[derive(Tabled)]
struct ActivateResponseRow {
    #[tabled(rename = "Success")]
    success: bool,
    #[tabled(rename = "Application ID")]
    application_id: String,
    #[tabled(rename = "Report")]
    report: String,
}

#[derive(Tabled)]
struct ActiveApplicationRow {
    #[tabled(rename = "Application ID")]
    application_id: String,
    #[tabled(rename = "Application")]
    app_name: String,
}

#[derive(Tabled)]
struct TerminateResponseRow {
    #[tabled(rename = "Success")]
    success: bool,
    #[tabled(rename = "Message")]
    message: String,
}

/// Parse a KEY=VALUE string into an EnvVar proto message.
fn parse_env_var(s: &str) -> Result<EnvVar, String> {
    let (key, value) = s
        .split_once('=')
        .ok_or_else(|| format!("env-var '{}' must be in KEY=VALUE format", s))?;
    Ok(EnvVar {
        key: key.to_string(),
        value: value.to_string(),
    })
}

pub async fn activate(
    address: String,
    output: OutputFormat,
    yaml_content: Option<String>,
    env_vars: Vec<String>,
    json: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ApplicationManagerClient::connect(address).await?;

    let request = if let Some(json_str) = json {
        let payload: ActivateApplicationRequestJson = serde_json::from_str(&json_str)?;
        ActivateApplicationRequest {
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
        ActivateApplicationRequest {
            yaml_content: yaml,
            env_vars: parsed_env_vars,
        }
    };

    let response = client.activate_application(request).await?.into_inner();

    match output {
        OutputFormat::Json => {
            let json_out = serde_json::json!({
                "success": response.success,
                "application_id": response.application_id,
                "report": response.report,
            });
            println!("{}", serde_json::to_string_pretty(&json_out)?);
        }
        OutputFormat::Table => {
            let rows: Vec<ActivateResponseRow> = response
                .report
                .iter()
                .enumerate()
                .map(|(i, line)| ActivateResponseRow {
                    success: if i == 0 { response.success } else { false },
                    application_id: if i == 0 {
                        response.application_id.clone()
                    } else {
                        String::new()
                    },
                    report: line.clone(),
                })
                .collect();

            let rows = if rows.is_empty() {
                vec![ActivateResponseRow {
                    success: response.success,
                    application_id: response.application_id,
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

pub async fn list(
    address: String,
    output: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ApplicationManagerClient::connect(address).await?;
    let response = client
        .list_active_applications(ListActiveApplicationsRequest {})
        .await?
        .into_inner();

    match output {
        OutputFormat::Json => {
            let apps: Vec<serde_json::Value> = response
                .applications
                .iter()
                .map(|a| {
                    serde_json::json!({
                        "application_id": a.application_id,
                        "app_name": a.app_name,
                    })
                })
                .collect();
            let json_out = serde_json::json!({
                "applications": apps,
            });
            println!("{}", serde_json::to_string_pretty(&json_out)?);
        }
        OutputFormat::Table => {
            let rows: Vec<ActiveApplicationRow> = response
                .applications
                .iter()
                .map(|a| ActiveApplicationRow {
                    application_id: a.application_id.clone(),
                    app_name: a.app_name.clone(),
                })
                .collect();
            println!("{}", Table::new(rows).with(Style::blank()));
        }
    }

    Ok(())
}

pub async fn terminate(
    address: String,
    output: OutputFormat,
    application_id: Option<String>,
    reason: Option<String>,
    json: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ApplicationManagerClient::connect(address).await?;

    let request = if let Some(json_str) = json {
        let payload: TerminateApplicationRequestJson = serde_json::from_str(&json_str)?;
        TerminateApplicationRequest {
            application_id: payload.application_id,
            reason: payload.reason,
        }
    } else {
        TerminateApplicationRequest {
            application_id: application_id
                .ok_or("--application-id is required when --json is not provided")?,
            reason: reason.unwrap_or_default(),
        }
    };

    let response = client.terminate_application(request).await?.into_inner();

    match output {
        OutputFormat::Json => {
            let json_out = serde_json::json!({
                "success": response.success,
                "message": response.message,
            });
            println!("{}", serde_json::to_string_pretty(&json_out)?);
        }
        OutputFormat::Table => {
            let rows = vec![TerminateResponseRow {
                success: response.success,
                message: response.message,
            }];
            println!("{}", Table::new(rows).with(Style::blank()));
        }
    }

    Ok(())
}
