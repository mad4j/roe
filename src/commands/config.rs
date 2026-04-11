use tabled::settings::Style;
use tabled::{Table, Tabled};

use crate::output::OutputFormat;
use crate::proto::configurable_application::{
    configurable_application_client::ConfigurableApplicationClient, ConfigureRequest, DataItem,
    DataType, QueryRequest,
};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Human-readable representation of a DataItem value.
fn value_display(item: &DataItem) -> String {
    use crate::proto::configurable_application::data_item::Value;
    match &item.value {
        Some(Value::BoolValue(v)) => v.to_string(),
        Some(Value::Int64Value(v)) => v.to_string(),
        Some(Value::DoubleValue(v)) => v.to_string(),
        Some(Value::StringValue(v)) => v.clone(),
        Some(Value::BytesValue(v)) => format!("<{} bytes>", v.len()),
        None => "<null>".to_string(),
    }
}

fn type_name(item: &DataItem) -> &'static str {
    match DataType::try_from(item.r#type) {
        Ok(DataType::Bool) => "bool",
        Ok(DataType::Int64) => "int64",
        Ok(DataType::Double) => "double",
        Ok(DataType::String) => "string",
        Ok(DataType::Bytes) => "bytes",
        Ok(DataType::Null) | Ok(DataType::Unspecified) | Err(_) => "null",
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
        r#type: type_name(item).to_string(),
        value: value_display(item),
    }
}

// ---------------------------------------------------------------------------
// Parse KEY=TYPE:VALUE strings into DataItems for Configure
// ---------------------------------------------------------------------------

/// Parse a string like `key=TYPE:value` or `key=null` into a DataItem.
/// Supported types: bool, int64, double, string, null.
fn parse_item(s: &str) -> Result<DataItem, String> {
    use crate::proto::configurable_application::data_item::Value;

    let (name, rest) = s
        .split_once('=')
        .ok_or_else(|| format!("invalid item '{}': expected NAME=TYPE:VALUE or NAME=null", s))?;

    if rest.eq_ignore_ascii_case("null") {
        return Ok(DataItem {
            name: name.to_string(),
            r#type: DataType::Null as i32,
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
                r#type: DataType::Bool as i32,
                value: Some(Value::BoolValue(v)),
            }
        }
        "int64" => {
            let v: i64 = val_str
                .parse()
                .map_err(|_| format!("cannot parse '{}' as int64", val_str))?;
            DataItem {
                name: name.to_string(),
                r#type: DataType::Int64 as i32,
                value: Some(Value::Int64Value(v)),
            }
        }
        "double" => {
            let v: f64 = val_str
                .parse()
                .map_err(|_| format!("cannot parse '{}' as double", val_str))?;
            DataItem {
                name: name.to_string(),
                r#type: DataType::Double as i32,
                value: Some(Value::DoubleValue(v)),
            }
        }
        "string" => DataItem {
            name: name.to_string(),
            r#type: DataType::String as i32,
            value: Some(Value::StringValue(val_str.to_string())),
        },
        other => return Err(format!("unknown type '{}'; use bool|int64|double|string|null", other)),
    };

    Ok(item)
}

// ---------------------------------------------------------------------------
// Query handler
// ---------------------------------------------------------------------------

/// Call ConfigurableApplication.Query.
/// `filter_names` – if non-empty, only those named items are requested.
pub async fn query(
    address: String,
    output: OutputFormat,
    filter_names: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ConfigurableApplicationClient::connect(address).await?;

    let items: Vec<DataItem> = filter_names
        .into_iter()
        .map(|n| DataItem {
            name: n,
            r#type: DataType::Null as i32,
            value: None,
        })
        .collect();

    let response = client
        .query(QueryRequest { items })
        .await?
        .into_inner();

    match output {
        OutputFormat::Json => {
            let json_items: Vec<serde_json::Value> = response
                .items
                .iter()
                .map(|i| {
                    serde_json::json!({
                        "name": i.name,
                        "type": type_name(i),
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
    address: String,
    output: OutputFormat,
    raw_items: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let items: Vec<DataItem> = raw_items
        .iter()
        .map(|s| parse_item(s))
        .collect::<Result<_, _>>()
        .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

    let mut client = ConfigurableApplicationClient::connect(address).await?;

    let response = client
        .configure(ConfigureRequest { items })
        .await?
        .into_inner();

    match output {
        OutputFormat::Json => {
            let rejected: Vec<serde_json::Value> = response
                .rejected_items
                .iter()
                .map(|i| {
                    serde_json::json!({
                        "name": i.name,
                        "type": type_name(i),
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
