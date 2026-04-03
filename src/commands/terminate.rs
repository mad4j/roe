use tabled::settings::Style;
use tabled::{Table, Tabled};

use crate::output::OutputFormat;
use crate::proto::managed_application::{
    TerminateRequest, managed_application_client::ManagedApplicationClient,
};

/// Table row used when rendering the terminate response
#[derive(Tabled)]
struct TerminateResponseRow {
    #[tabled(rename = "Success")]
    success: bool,
    #[tabled(rename = "Message")]
    message: String,
}

pub async fn handle(
    address: String,
    output: OutputFormat,
    reason: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ManagedApplicationClient::connect(address).await?;

    let response = client
        .terminate(TerminateRequest {
            reason: reason.unwrap_or_default(),
        })
        .await?
        .into_inner();

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
