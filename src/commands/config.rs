use serde::{Deserialize, Serialize};
use tabled::settings::Style;
use tabled::{Table, Tabled};

use crate::client;
use crate::output::OutputFormat;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Supported data types for a configuration item.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    Unspecified,
    Bool,
    Int64,
    Double,
    String,
    Bytes,
    Null,
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DataType::Bool => "bool",
            DataType::Int64 => "int64",
            DataType::Double => "double",
            DataType::String => "string",
            DataType::Bytes => "bytes",
            DataType::Null | DataType::Unspecified => "null",
        };
        f.write_str(s)
    }
}

/// Typed value for a configuration item.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum DataValue {
    Bool(bool),
    Int64(i64),
    Double(f64),
    String(String),
    Bytes(Vec<u8>),
}

/// A single named configuration item with an optional typed value.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataItem {
    pub name: String,
    #[serde(rename = "type")]
    pub data_type: DataType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<DataValue>,
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Human-readable representation of a DataItem value.
fn value_display(item: &DataItem) -> String {
    match &item.value {
        Some(DataValue::Bool(v)) => v.to_string(),
        Some(DataValue::Int64(v)) => v.to_string(),
        Some(DataValue::Double(v)) => v.to_string(),
        Some(DataValue::String(v)) => v.clone(),
        Some(DataValue::Bytes(v)) => format!("<{} bytes>", v.len()),
        None => "<null>".to_string(),
    }
}

#[derive(Tabled)]
struct DataItemRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Type")]
    r#type: String,
    #[tabled(rename = "Value")]
    value: String,
}

fn to_row(item: &DataItem) -> DataItemRow {
    DataItemRow {
        name: item.name.clone(),
        r#type: item.data_type.to_string(),
        value: value_display(item),
    }
}

// ---------------------------------------------------------------------------
// Parse KEY=TYPE:VALUE strings into DataItems for Configure
// ---------------------------------------------------------------------------

/// Parse a string like `key=TYPE:value` or `key=null` into a DataItem.
/// Supported types: bool, int64, double, string, null.
fn parse_item(s: &str) -> Result<DataItem, String> {
    let (name, rest) = s
        .split_once('=')
        .ok_or_else(|| format!("invalid item '{}': expected NAME=TYPE:VALUE or NAME=null", s))?;

    if rest.eq_ignore_ascii_case("null") {
        return Ok(DataItem {
            name: name.to_string(),
            data_type: DataType::Null,
            value: None,
        });
    }

    let (type_str, val_str) = rest.split_once(':').ok_or_else(|| {
        format!(
            "invalid item '{}': expected NAME=TYPE:VALUE (e.g. count=int64:42)",
            s
        )
    })?;

    let item = match type_str.to_lowercase().as_str() {
        "bool" => {
            let v: bool = val_str
                .parse()
                .map_err(|_| format!("cannot parse '{}' as bool", val_str))?;
            DataItem {
                name: name.to_string(),
                data_type: DataType::Bool,
                value: Some(DataValue::Bool(v)),
            }
        }
        "int64" => {
            let v: i64 = val_str
                .parse()
                .map_err(|_| format!("cannot parse '{}' as int64", val_str))?;
            DataItem {
                name: name.to_string(),
                data_type: DataType::Int64,
                value: Some(DataValue::Int64(v)),
            }
        }
        "double" => {
            let v: f64 = val_str
                .parse()
                .map_err(|_| format!("cannot parse '{}' as double", val_str))?;
            DataItem {
                name: name.to_string(),
                data_type: DataType::Double,
                value: Some(DataValue::Double(v)),
            }
        }
        "string" => DataItem {
            name: name.to_string(),
            data_type: DataType::String,
            value: Some(DataValue::String(val_str.to_string())),
        },
        other => return Err(format!("unknown type '{}'; use bool|int64|double|string|null", other)),
    };

    Ok(item)
}

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug)]
struct QueryRequest {
    items: Vec<DataItem>,
}

#[derive(Serialize, Deserialize, Debug)]
struct QueryResponse {
    items: Vec<DataItem>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ConfigureRequest {
    items: Vec<DataItem>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ConfigureResponse {
    success: bool,
    message: String,
    rejected_items: Vec<DataItem>,
}

// ---------------------------------------------------------------------------
// Query handler
// ---------------------------------------------------------------------------

/// Call ConfigurableApplication.Query.
/// `filter_names` – if non-empty, only those named items are requested.
pub async fn query(
    peer: String,
    output: OutputFormat,
    filter_names: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let items: Vec<DataItem> = filter_names
        .into_iter()
        .map(|n| DataItem {
            name: n,
            data_type: DataType::Null,
            value: None,
        })
        .collect();

    let response: QueryResponse =
        client::call(&peer, "ConfigurableApplication", "Query", &QueryRequest { items }).await?;

    match output {
        OutputFormat::Json => {
            let json_items: Vec<serde_json::Value> = response
                .items
                .iter()
                .map(|i| {
                    serde_json::json!({
                        "name": i.name,
                        "type": i.data_type.to_string(),
                        "value": value_display(i),
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_items)?);
        }
        OutputFormat::Table => {
            let rows: Vec<DataItemRow> = response.items.iter().map(to_row).collect();
            println!("{}", Table::new(rows).with(Style::blank()));
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Configure handler
// ---------------------------------------------------------------------------

/// Call ConfigurableApplication.Configure.
/// `raw_items` – strings in `NAME=TYPE:VALUE` or `NAME=null` format.
pub async fn configure(
    peer: String,
    output: OutputFormat,
    raw_items: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let items: Vec<DataItem> = raw_items
        .iter()
        .map(|s| parse_item(s))
        .collect::<Result<_, _>>()
        .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

    let response: ConfigureResponse =
        client::call(&peer, "ConfigurableApplication", "Configure", &ConfigureRequest { items }).await?;

    match output {
        OutputFormat::Json => {
            let rejected: Vec<serde_json::Value> = response
                .rejected_items
                .iter()
                .map(|i| {
                    serde_json::json!({
                        "name": i.name,
                        "type": i.data_type.to_string(),
                        "value": value_display(i),
                    })
                })
                .collect();
            let json_out = serde_json::json!({
                "success": response.success,
                "message": response.message,
                "rejected_items": rejected,
            });
            println!("{}", serde_json::to_string_pretty(&json_out)?);
        }
        OutputFormat::Table => {
            #[derive(Tabled)]
            struct ResultRow {
                #[tabled(rename = "Success")]
                success: bool,
                #[tabled(rename = "Message")]
                message: String,
            }
            let rows = vec![ResultRow {
                success: response.success,
                message: response.message,
            }];
            println!("{}", Table::new(rows).with(Style::blank()));

            if !response.rejected_items.is_empty() {
                println!("\nRejected items:");
                let rejected_rows: Vec<DataItemRow> =
                    response.rejected_items.iter().map(to_row).collect();
                println!("{}", Table::new(rejected_rows).with(Style::blank()));
            }
        }
    }

    Ok(())
}
