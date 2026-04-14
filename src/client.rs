use hdds::{Participant, rpc::ServiceClient};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// Default timeout for DDS-RPC calls.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Build a DDS [`Participant`] named `"roe-cli"`.
///
/// When `peer` is non-empty the address is registered as a static unicast peer
/// (useful when multicast discovery is unavailable, e.g. across subnets).
/// When `peer` is empty the participant relies on RTPS multicast discovery.
pub fn build_participant(peer: &str) -> Result<Arc<Participant>, hdds::Error> {
    let mut builder = Participant::builder("roe-cli");
    if !peer.is_empty() {
        builder = builder.add_static_peer(peer);
    }
    builder.build()
}

/// Call a single DDS-RPC operation on `service_name`.
///
/// The request is serialised as a JSON envelope `{"op": "<op>", "data": <req>}`
/// and the reply is deserialised from plain JSON.
pub async fn call<Req, Res>(
    peer: &str,
    service_name: &str,
    op: &str,
    request: &Req,
) -> Result<Res, Box<dyn std::error::Error>>
where
    Req: Serialize,
    Res: for<'de> Deserialize<'de>,
{
    let participant = build_participant(peer)?;
    let client = ServiceClient::new(&participant, service_name)?;

    let envelope = serde_json::json!({
        "op": op,
        "data": serde_json::to_value(request)?,
    });
    let payload = serde_json::to_vec(&envelope)?;

    let reply_bytes = client.call_raw(&payload, DEFAULT_TIMEOUT).await
        .map_err(|e| format!("DDS-RPC call to {}/{} failed: {}", service_name, op, e))?;

    let response: Res = serde_json::from_slice(&reply_bytes)?;
    Ok(response)
}
