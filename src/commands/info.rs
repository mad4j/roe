use serde::{Deserialize, Serialize};
use tabled::{Table, Tabled};
use tabled::settings::Style;

use crate::client;
use crate::output::OutputFormat;

/// Request payload for the Info operation (no parameters required).
#[derive(Serialize, Deserialize, Debug)]
struct InfoRequest {}

/// A listening address paired with the services reachable at that address.
#[derive(Serialize, Deserialize, Debug)]
struct ListeningAddress {
    address: String,
    services: Vec<String>,
}

/// Response payload for the Info operation.
#[derive(Serialize, Deserialize, Debug)]
struct InfoResponse {
    app_name: String,
    listening_addresses: Vec<ListeningAddress>,
}

/// Table row used when rendering the info response
#[derive(Tabled)]
struct ListeningAddressRow {
    #[tabled(rename = "Address")]
    address: String,
    #[tabled(rename = "Services")]
    services: String,
}

pub async fn handle(
    peer: String,
    output: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let response: InfoResponse =
        client::call(&peer, "ManagedApplication", "Info", &InfoRequest {}).await?;

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
                    services: a.services.join("\n"),
                })
                .collect();
            println!("{}", Table::new(rows).with(Style::blank()));
        }
    }

    Ok(())
}
