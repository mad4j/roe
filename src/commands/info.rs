use tabled::{Table, Tabled};

use crate::output::OutputFormat;
use crate::proto::managed_application::{InfoRequest, managed_application_client::ManagedApplicationClient};

/// Table row used when rendering the info response
#[derive(Tabled)]
struct ListeningAddressRow {
    #[tabled(rename = "Address")]
    address: String,
    #[tabled(rename = "Services")]
    services: String,
}

pub async fn handle(
    address: String,
    output: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ManagedApplicationClient::connect(address).await?;
    let response = client.info(InfoRequest {}).await?.into_inner();

    match output {
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

    Ok(())
}
