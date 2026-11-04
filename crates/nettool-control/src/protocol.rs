//! JSON-lines encoding for the daemon control channel.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use crate::json::Json;
use crate::{
    ControlCommand, ControlResponse, MacLease, MacLeaseState, SessionInfo, SessionState,
    API_VERSION,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlRequest {
    pub api_version: u32,
    pub token: String,
    pub command: ControlCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolError {
    message: String,
}

impl ProtocolError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for ProtocolError {}

pub fn parse_request(line: &str) -> Result<ControlRequest, ProtocolError> {
    let value =
        Json::parse(line).map_err(|error| ProtocolError::new(format!("invalid JSON: {error}")))?;
    let object = value
        .as_object()
        .ok_or_else(|| ProtocolError::new("request must be a JSON object"))?;

    let api_version = required_u64(object, "api_version")?;
    let api_version = u32::try_from(api_version)
        .map_err(|_| ProtocolError::new("api_version is out of range"))?;
    let token = required_str(object, "token")?.to_string();
    let command_name = required_str(object, "command")?;

    let command = match command_name {
        "start_proxy" => ControlCommand::StartProxy {
            listen: required_str(object, "listen")?.to_string(),
            target: required_str(object, "target")?.to_string(),
            protocol: required_str(object, "protocol")?.to_string(),
        },
        "stop_proxy" => ControlCommand::StopProxy {
            session: required_u64(object, "session")?,
        },
        "list_sessions" => ControlCommand::ListSessions,
        "subscribe" => ControlCommand::Subscribe {
            from_event: optional_u64(object, "from_event")?.unwrap_or(0),
        },
        "acquire_mac" => ControlCommand::AcquireMac {
            interface: required_str(object, "interface")?.to_string(),
            requested: required_str(object, "requested")?.to_string(),
            dry_run: optional_bool(object, "dry_run")?.unwrap_or(false),
        },
        "release_mac" => ControlCommand::ReleaseMac {
            lease_id: required_u64(object, "lease_id")?,
        },
        "list_leases" => ControlCommand::ListLeases,
        "shutdown" => ControlCommand::Shutdown,
        "shutdown_helper" => ControlCommand::ShutdownHelper,
        other => return Err(ProtocolError::new(format!("unknown command '{other}'"))),
    };

    Ok(ControlRequest {
        api_version,
        token,
        command,
    })
}

pub fn response_json(response: &ControlResponse) -> String {
    let mut object = BTreeMap::new();
    object.insert("api_version".to_string(), Json::Number(API_VERSION as f64));
    match response {
        ControlResponse::Ok => {
            object.insert("ok".to_string(), Json::Bool(true));
        }
        ControlResponse::SessionId(session) => {
            object.insert("ok".to_string(), Json::Bool(true));
            object.insert("session_id".to_string(), Json::Number(*session as f64));
        }
        ControlResponse::LeaseId(lease) => {
            object.insert("ok".to_string(), Json::Bool(true));
            object.insert("lease_id".to_string(), Json::Number(*lease as f64));
        }
        ControlResponse::Sessions(sessions) => {
            object.insert("ok".to_string(), Json::Bool(true));
            object.insert(
                "sessions".to_string(),
                Json::Array(sessions.iter().map(session_json).collect()),
            );
        }
        ControlResponse::Leases(leases) => {
            object.insert("ok".to_string(), Json::Bool(true));
            object.insert(
                "leases".to_string(),
                Json::Array(leases.iter().map(lease_json).collect()),
            );
        }
        ControlResponse::Error(message) => {
            object.insert("ok".to_string(), Json::Bool(false));
            object.insert("error".to_string(), Json::String(message.clone()));
        }
    }
    Json::Object(object).to_json_string()
}

fn session_json(session: &SessionInfo) -> Json {
    let mut object = BTreeMap::new();
    object.insert("id".to_string(), Json::Number(session.id as f64));
    object.insert("listen".to_string(), Json::String(session.listen.clone()));
    object.insert("target".to_string(), Json::String(session.target.clone()));
    object.insert(
        "protocol".to_string(),
        Json::String(session.protocol.clone()),
    );
    object.insert(
        "state".to_string(),
        Json::String(session_state(session.state).to_string()),
    );
    Json::Object(object)
}

fn lease_json(lease: &MacLease) -> Json {
    let mut object = BTreeMap::new();
    object.insert("id".to_string(), Json::Number(lease.id as f64));
    object.insert(
        "interface".to_string(),
        Json::String(lease.interface.clone()),
    );
    object.insert("original".to_string(), Json::String(lease.original.clone()));
    object.insert("active".to_string(), Json::String(lease.active.clone()));
    object.insert(
        "state".to_string(),
        Json::String(lease_state(lease.state).to_string()),
    );
    Json::Object(object)
}

fn session_state(state: SessionState) -> &'static str {
    match state {
        SessionState::Started => "started",
        SessionState::Stopped => "stopped",
        SessionState::Error => "error",
    }
}

fn mac_lease_state(state: MacLeaseState) -> &'static str {
    match state {
        MacLeaseState::Active => "active",
        MacLeaseState::Releasing => "releasing",
        MacLeaseState::Released => "released",
        MacLeaseState::Error => "error",
    }
}

fn lease_state(state: MacLeaseState) -> &'static str {
    mac_lease_state(state)
}

fn required_str<'a>(
    object: &'a BTreeMap<String, Json>,
    key: &str,
) -> Result<&'a str, ProtocolError> {
    object
        .get(key)
        .and_then(Json::as_str)
        .ok_or_else(|| ProtocolError::new(format!("missing or invalid string field '{key}'")))
}

fn required_u64(object: &BTreeMap<String, Json>, key: &str) -> Result<u64, ProtocolError> {
    object
        .get(key)
        .and_then(Json::as_u64)
        .ok_or_else(|| ProtocolError::new(format!("missing or invalid integer field '{key}'")))
}

fn optional_u64(object: &BTreeMap<String, Json>, key: &str) -> Result<Option<u64>, ProtocolError> {
    match object.get(key) {
        None => Ok(None),
        Some(value) => value
            .as_u64()
            .map(Some)
            .ok_or_else(|| ProtocolError::new(format!("invalid integer field '{key}'"))),
    }
}

fn optional_bool(
    object: &BTreeMap<String, Json>,
    key: &str,
) -> Result<Option<bool>, ProtocolError> {
    match object.get(key) {
        None => Ok(None),
        Some(value) => value
            .as_bool()
            .map(Some)
            .ok_or_else(|| ProtocolError::new(format!("invalid boolean field '{key}'"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_list_leases_request() {
        let request =
            parse_request(r#"{"api_version":1,"token":"secret","command":"list_leases"}"#).unwrap();
        assert_eq!(request.api_version, 1);
        assert_eq!(request.token, "secret");
        assert_eq!(request.command, ControlCommand::ListLeases);
    }

    #[test]
    fn parses_acquire_request() {
        let request = parse_request(
            r#"{"api_version":1,"token":"s","command":"acquire_mac","interface":"en0","requested":"02:00:00:00:00:01","dry_run":true}"#,
        )
        .unwrap();
        assert_eq!(
            request.command,
            ControlCommand::AcquireMac {
                interface: "en0".to_string(),
                requested: "02:00:00:00:00:01".to_string(),
                dry_run: true,
            }
        );
    }

    #[test]
    fn rejects_bad_requests() {
        for input in [
            r#"{"api_version":1,"command":"list_leases"}"#,
            r#"{"api_version":1,"token":"s","command":"nope"}"#,
            r#"{"api_version":"1","token":"s","command":"list_leases"}"#,
        ] {
            assert!(parse_request(input).is_err(), "expected invalid: {input}");
        }
    }

    #[test]
    fn serializes_lease_response() {
        let response = ControlResponse::Leases(vec![MacLease {
            id: 3,
            interface: "en0".to_string(),
            original: "aa:bb:cc:dd:ee:ff".to_string(),
            active: "02:00:00:00:00:01".to_string(),
            state: MacLeaseState::Active,
        }]);
        let text = response_json(&response);
        let value = Json::parse(&text).unwrap();
        assert_eq!(value.get("ok").and_then(Json::as_bool), Some(true));
        assert_eq!(value.get("api_version").and_then(Json::as_u64), Some(1));
        let leases = value.get("leases").and_then(Json::as_array).unwrap();
        assert_eq!(leases.len(), 1);
        assert_eq!(
            leases[0].get("interface").and_then(Json::as_str),
            Some("en0")
        );
    }
}
