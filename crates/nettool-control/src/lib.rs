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

//! Control types shared by the daemon, CLI and UI clients.
//!
//! This crate has no I/O and no transport. It defines the message model and
//! the MAC lease state machine so the process boundary can be tested without
//! touching a real network interface.
//!
//! The UI never links the proxy or capture crates. It talks to the daemon,
//! and the daemon talks to the privileged helper. See docs/PROCESS-MODEL.md.

pub mod json;
pub mod protocol;

pub const API_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Started,
    Stopped,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionInfo {
    pub id: u64,
    pub listen: String,
    pub target: String,
    pub protocol: String,
    pub state: SessionState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacLeaseState {
    Active,
    Releasing,
    Released,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacLease {
    pub id: u64,
    pub interface: String,
    /// Identity captured before the lease, used by the helper to restore.
    pub original: String,
    /// Identity currently applied while the lease is active.
    pub active: String,
    pub state: MacLeaseState,
}

#[derive(Debug, Default)]
pub struct MacLeaseManager {
    next_id: u64,
    leases: Vec<MacLease>,
}

impl MacLeaseManager {
    /// Acquire a lease. Idempotent per interface: an existing active lease is
    /// returned instead of creating a second one.
    pub fn acquire(&mut self, interface: &str, original: &str, requested: &str) -> u64 {
        if let Some(existing) = self
            .leases
            .iter()
            .find(|lease| lease.interface == interface && lease.state == MacLeaseState::Active)
        {
            return existing.id;
        }
        self.next_id += 1;
        let id = self.next_id;
        self.leases.push(MacLease {
            id,
            interface: interface.to_string(),
            original: original.to_string(),
            active: requested.to_string(),
            state: MacLeaseState::Active,
        });
        id
    }

    /// First half of an explicit release. The helper restores the original
    /// identity between begin_release and confirm_release.
    pub fn begin_release(&mut self, lease_id: u64) -> bool {
        if let Some(lease) = self
            .leases
            .iter_mut()
            .find(|lease| lease.id == lease_id && lease.state == MacLeaseState::Active)
        {
            lease.state = MacLeaseState::Releasing;
            true
        } else {
            false
        }
    }

    /// Second half of an explicit release, after restore verification.
    pub fn confirm_release(&mut self, lease_id: u64) -> bool {
        if let Some(lease) = self
            .leases
            .iter_mut()
            .find(|lease| lease.id == lease_id && lease.state == MacLeaseState::Releasing)
        {
            lease.state = MacLeaseState::Released;
            true
        } else {
            false
        }
    }

    /// Convenience for callers that do the restore synchronously.
    pub fn release(&mut self, lease_id: u64) -> bool {
        self.begin_release(lease_id) && self.confirm_release(lease_id)
    }

    /// Release every active lease. Used by explicit helper shutdown only, not
    /// by client disconnect.
    pub fn release_all(&mut self) -> Vec<u64> {
        let mut released = Vec::new();
        for lease in &mut self.leases {
            if lease.state == MacLeaseState::Active {
                lease.state = MacLeaseState::Released;
                released.push(lease.id);
            }
        }
        released
    }

    pub fn active_leases(&self) -> Vec<&MacLease> {
        self.leases
            .iter()
            .filter(|lease| lease.state == MacLeaseState::Active)
            .collect()
    }

    pub fn get(&self, lease_id: u64) -> Option<&MacLease> {
        self.leases.iter().find(|lease| lease.id == lease_id)
    }

    pub fn leases(&self) -> &[MacLease] {
        &self.leases
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketSummary {
    pub id: u64,
    pub direction: String,
    pub protocol: String,
    pub length: usize,
    pub dropped: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlCommand {
    StartProxy {
        listen: String,
        target: String,
        protocol: String,
    },
    StopProxy {
        session: u64,
    },
    ListSessions,
    Subscribe {
        from_event: u64,
    },
    AcquireMac {
        interface: String,
        requested: String,
        dry_run: bool,
    },
    ReleaseMac {
        lease_id: u64,
    },
    ListLeases,
    Shutdown,
    ShutdownHelper,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlResponse {
    Ok,
    SessionId(u64),
    LeaseId(u64),
    Sessions(Vec<SessionInfo>),
    Leases(Vec<MacLease>),
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlEvent {
    Packet(PacketSummary),
    Session {
        id: u64,
        state: SessionState,
    },
    MacLease {
        lease: MacLease,
        state: MacLeaseState,
    },
    Helper {
        running: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquire_is_idempotent_per_interface() {
        let mut manager = MacLeaseManager::default();
        let first = manager.acquire("en0", "aa:bb:cc:dd:ee:ff", "02:00:00:00:00:01");
        let second = manager.acquire("en0", "aa:bb:cc:dd:ee:ff", "02:00:00:00:00:02");
        assert_eq!(first, second);
        assert_eq!(manager.active_leases().len(), 1);
        assert_eq!(manager.active_leases()[0].active, "02:00:00:00:00:01");
    }

    #[test]
    fn explicit_release_transitions() {
        let mut manager = MacLeaseManager::default();
        let id = manager.acquire("en0", "aa:bb:cc:dd:ee:ff", "02:00:00:00:00:01");
        assert!(manager.begin_release(id));
        assert_eq!(manager.get(id).unwrap().state, MacLeaseState::Releasing);
        assert!(manager.confirm_release(id));
        assert_eq!(manager.get(id).unwrap().state, MacLeaseState::Released);
        assert!(manager.active_leases().is_empty());
    }

    #[test]
    fn release_is_explicit_and_not_idempotent() {
        let mut manager = MacLeaseManager::default();
        let id = manager.acquire("en0", "aa:bb:cc:dd:ee:ff", "02:00:00:00:00:01");
        assert!(manager.release(id));
        assert!(!manager.release(id));
    }

    #[test]
    fn shutdown_releases_all_active_leases() {
        let mut manager = MacLeaseManager::default();
        let one = manager.acquire("en0", "aa:bb:cc:dd:ee:ff", "02:00:00:00:00:01");
        let two = manager.acquire("en1", "aa:bb:cc:dd:ee:00", "02:00:00:00:00:02");
        let released = manager.release_all();
        assert_eq!(released, vec![one, two]);
        assert!(manager.active_leases().is_empty());
    }

    #[test]
    fn disconnect_does_not_release() {
        // There is deliberately no "client disconnected" method. A lease only
        // ends through begin_release / release / release_all.
        let mut manager = MacLeaseManager::default();
        let id = manager.acquire("en0", "aa:bb:cc:dd:ee:ff", "02:00:00:00:00:01");
        assert_eq!(manager.active_leases().len(), 1);
        assert_eq!(manager.get(id).unwrap().state, MacLeaseState::Active);
    }

    #[test]
    fn control_message_roundtrip_types() {
        let command = ControlCommand::AcquireMac {
            interface: "en0".to_string(),
            requested: "02:00:00:00:00:01".to_string(),
            dry_run: true,
        };
        assert_eq!(command.clone(), command);
        let event = ControlEvent::Helper { running: true };
        assert_eq!(event.clone(), event);
    }
}
