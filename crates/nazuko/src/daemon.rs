// 本文件由 SilentStudio 监管与负责。
// 本文件属于 SilentCodeTeams 产品 Nazu&Ko 的源码部分
// 本项目由 SilentCodeTeams 对法律、版权事宜及相关咨询承担责任。
// 版权所有 © SilentStudio。
// 以 AGPL-3.0-or-later 授权。
// This document is supervised and responsible by SilentStudio.
// This file is part of the source code of the SilentCodeTeams product Nazu&Ko
// SilentCodeTeams is responsible for legal and copyright issues and related consultation in this project.
// Copyright © SilentStudio.
// Licensed as AGPL-3.0-or-later.

//! Local daemon control server.
//!
//! Phase 2 transport: loopback TCP, a per-process bearer token in a 0600 state
//! file, and one JSON object per line. Proxy session commands are intentionally
//! not wired yet; this slice proves authentication, framing and MAC lease
//! control without exposing a remote interface.

use std::collections::hash_map::RandomState;
use std::fs::{self, OpenOptions};
use std::hash::{BuildHasher, Hasher};
use std::io::{self, BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use nettool_control::protocol::{parse_request, response_json};
use nettool_control::{ControlCommand, ControlResponse, MacLeaseManager, API_VERSION};

const MAX_LINE_BYTES: usize = 1_048_576;

#[derive(Clone)]
pub struct Daemon {
    listener: Arc<TcpListener>,
    token: Arc<String>,
    state_path: Arc<PathBuf>,
    running: Arc<AtomicBool>,
    state: Arc<Mutex<ServerState>>,
}

struct ServerState {
    leases: MacLeaseManager,
}

impl Daemon {
    pub fn bind(bind: SocketAddr, state_path: PathBuf) -> io::Result<Self> {
        if !bind.ip().is_loopback() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "control server must bind to a loopback address",
            ));
        }
        let listener = TcpListener::bind(bind)?;
        listener.set_nonblocking(true)?;
        Ok(Self {
            listener: Arc::new(listener),
            token: Arc::new(generate_token()),
            state_path: Arc::new(state_path),
            running: Arc::new(AtomicBool::new(true)),
            state: Arc::new(Mutex::new(ServerState {
                leases: MacLeaseManager::default(),
            })),
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.listener.local_addr()
    }

    #[cfg(test)]
    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn run(self) -> io::Result<()> {
        self.write_state()?;
        while self.running.load(Ordering::SeqCst) {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    let daemon = self.clone();
                    thread::spawn(move || {
                        if let Err(error) = daemon.handle_client(stream) {
                            eprintln!("[control] connection error: {error}");
                        }
                    });
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(25));
                }
                Err(error) => {
                    let _ = fs::remove_file(self.state_path.as_ref());
                    return Err(error);
                }
            }
        }
        let _ = fs::remove_file(self.state_path.as_ref());
        Ok(())
    }

    fn write_state(&self) -> io::Result<()> {
        let address = self.local_addr()?;
        if let Some(parent) = self.state_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let mut options = OpenOptions::new();
        options.create(true).write(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(self.state_path.as_ref())?;
        writeln!(file, "port={}", address.port())?;
        writeln!(file, "token={}", self.token.as_str())?;
        file.flush()
    }

    fn handle_client(&self, stream: TcpStream) -> io::Result<()> {
        stream.set_nonblocking(false)?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut writer = stream;
        let mut line = String::new();

        loop {
            line.clear();
            let read = reader.read_line(&mut line)?;
            if read == 0 {
                break;
            }
            if line.len() > MAX_LINE_BYTES {
                send_response(
                    &mut writer,
                    &ControlResponse::Error("request line too large".to_string()),
                )?;
                break;
            }

            let request = match parse_request(line.trim()) {
                Ok(request) => request,
                Err(error) => {
                    send_response(&mut writer, &ControlResponse::Error(error.to_string()))?;
                    continue;
                }
            };

            if request.token != self.token.as_str() {
                send_response(
                    &mut writer,
                    &ControlResponse::Error("invalid control token".to_string()),
                )?;
                break;
            }
            if request.api_version != API_VERSION {
                send_response(
                    &mut writer,
                    &ControlResponse::Error(format!(
                        "unsupported api_version {}; daemon speaks {}",
                        request.api_version, API_VERSION
                    )),
                )?;
                break;
            }

            let shutdown = matches!(&request.command, ControlCommand::Shutdown);
            let response = self.handle_command(request.command);
            send_response(&mut writer, &response)?;
            if shutdown {
                break;
            }
        }

        Ok(())
    }

    fn handle_command(&self, command: ControlCommand) -> ControlResponse {
        match command {
            ControlCommand::ListSessions => ControlResponse::Sessions(Vec::new()),
            ControlCommand::ListLeases => {
                let state = self
                    .state
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                ControlResponse::Leases(state.leases.leases().to_vec())
            }
            ControlCommand::AcquireMac {
                interface,
                requested,
                dry_run,
            } => {
                if !dry_run {
                    return ControlResponse::Error(
                        "MAC helper is not available; use dry_run=true".to_string(),
                    );
                }
                if interface.trim().is_empty() || requested.trim().is_empty() {
                    return ControlResponse::Error(
                        "interface and requested identity are required".to_string(),
                    );
                }
                let mut state = self
                    .state
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let lease = state.leases.acquire(&interface, "(dry-run)", &requested);
                ControlResponse::LeaseId(lease)
            }
            ControlCommand::ReleaseMac { lease_id } => {
                let mut state = self
                    .state
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                if state.leases.release(lease_id) {
                    ControlResponse::Ok
                } else {
                    ControlResponse::Error(format!("active lease {lease_id} not found"))
                }
            }
            ControlCommand::Shutdown => {
                self.running.store(false, Ordering::SeqCst);
                ControlResponse::Ok
            }
            _ => ControlResponse::Error(
                "command is not implemented in this daemon slice".to_string(),
            ),
        }
    }
}

fn send_response(writer: &mut TcpStream, response: &ControlResponse) -> io::Result<()> {
    writer.write_all(response_json(response).as_bytes())?;
    writer.write_all(b"\n")?;
    writer.flush()
}

fn generate_token() -> String {
    let mut token = String::with_capacity(32);
    for round in 0u8..2 {
        let mut hasher = RandomState::new().build_hasher();
        hasher.write_u128(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        );
        hasher.write_u32(std::process::id());
        hasher.write_u8(round);
        token.push_str(&format!("{:016x}", hasher.finish()));
    }
    token
}

#[cfg(test)]
mod tests {
    use super::*;
    use nettool_control::json::Json;

    fn temporary_state_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "nazuko-test-{}-{}-{}.state",
            std::process::id(),
            name,
            nanos
        ))
    }

    fn wait_for_state(path: &std::path::Path) {
        for _ in 0..200 {
            if path.exists() {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("state file was not written");
    }

    fn write_line(stream: &mut TcpStream, line: &str) {
        stream.write_all(line.as_bytes()).unwrap();
        stream.write_all(b"\n").unwrap();
        stream.flush().unwrap();
    }

    fn read_response(reader: &mut BufReader<TcpStream>) -> Json {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        Json::parse(line.trim()).unwrap()
    }

    #[test]
    fn control_server_handles_lease_lifecycle() {
        let path = temporary_state_path("leases");
        let daemon = Daemon::bind("127.0.0.1:0".parse().unwrap(), path.clone()).unwrap();
        let token = daemon.token().to_string();
        let address = daemon.local_addr().unwrap();
        let server = daemon.clone();
        let handle = thread::spawn(move || server.run());

        wait_for_state(&path);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }

        let mut stream = TcpStream::connect(address).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());

        write_line(
            &mut stream,
            &format!(r#"{{"api_version":1,"token":"{token}","command":"list_leases"}}"#),
        );
        let response = read_response(&mut reader);
        assert_eq!(response.get("ok").and_then(Json::as_bool), Some(true));
        assert_eq!(
            response
                .get("leases")
                .and_then(Json::as_array)
                .map(|leases| leases.len()),
            Some(0)
        );

        write_line(
            &mut stream,
            &format!(
                r#"{{"api_version":1,"token":"{token}","command":"acquire_mac","interface":"en0","requested":"02:00:00:00:00:01","dry_run":true}}"#
            ),
        );
        let response = read_response(&mut reader);
        let lease_id = response
            .get("lease_id")
            .and_then(Json::as_u64)
            .expect("lease id");
        assert_eq!(lease_id, 1);

        write_line(
            &mut stream,
            &format!(r#"{{"api_version":1,"token":"{token}","command":"list_leases"}}"#),
        );
        let response = read_response(&mut reader);
        let leases = response.get("leases").and_then(Json::as_array).unwrap();
        assert_eq!(leases.len(), 1);
        assert_eq!(
            leases[0].get("interface").and_then(Json::as_str),
            Some("en0")
        );

        write_line(
            &mut stream,
            &format!(
                r#"{{"api_version":1,"token":"{token}","command":"release_mac","lease_id":{lease_id}}}"#
            ),
        );
        let response = read_response(&mut reader);
        assert_eq!(response.get("ok").and_then(Json::as_bool), Some(true));

        write_line(
            &mut stream,
            &format!(r#"{{"api_version":1,"token":"{token}","command":"shutdown"}}"#),
        );
        let response = read_response(&mut reader);
        assert_eq!(response.get("ok").and_then(Json::as_bool), Some(true));

        handle.join().unwrap().unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn control_server_rejects_invalid_token() {
        let path = temporary_state_path("token");
        let daemon = Daemon::bind("127.0.0.1:0".parse().unwrap(), path.clone()).unwrap();
        let address = daemon.local_addr().unwrap();
        let server = daemon.clone();
        let handle = thread::spawn(move || server.run());
        wait_for_state(&path);

        let mut stream = TcpStream::connect(address).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        write_line(
            &mut stream,
            r#"{"api_version":1,"token":"wrong","command":"list_leases"}"#,
        );
        let response = read_response(&mut reader);
        assert_eq!(response.get("ok").and_then(Json::as_bool), Some(false));

        drop(reader);
        drop(stream);
        let mut stream = TcpStream::connect(address).unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        write_line(
            &mut stream,
            &format!(
                r#"{{"api_version":1,"token":"{}","command":"shutdown"}}"#,
                daemon.token()
            ),
        );
        let _ = read_response(&mut reader);
        handle.join().unwrap().unwrap();
    }
}
