use serde::{Deserialize, Serialize};
use tabled::settings::Style;
use tabled::{Table, Tabled};

use crate::client;
use crate::output::OutputFormat;

/// Request payload for the Terminate operation.
#[derive(Serialize, Deserialize, Debug)]
struct TerminateRequest {
    reason: String,
}

/// Response payload for the Terminate operation.
#[derive(Serialize, Deserialize, Debug)]
struct TerminateResponse {
    success: bool,
    message: String,
}

/// Table row used when rendering the terminate response
#[derive(Tabled)]
struct TerminateResponseRow {
    #[tabled(rename = "Success")]
    success: bool,
    #[tabled(rename = "Message")]
    message: String,
}

pub async fn handle(
    peer: String,
    output: OutputFormat,
    reason: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = TerminateRequest {
        reason: reason.unwrap_or_default(),
    };

    let response: TerminateResponse =
        client::call(&peer, "ManagedApplication", "Terminate", &request).await?;

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
