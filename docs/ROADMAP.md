# Nazu&Ko roadmap

Status: active  
Date: 2026-11-04  
Repository: `Nazu-and-Ko`  
Product: `Nazu&Ko`  
License: AGPL-3.0-or-later

> We don't sleep because packets never do.

## 1. Fixed decisions

- GitHub and local repository name is exactly `Nazu-and-Ko`.
- Product name is `Nazu&Ko`; CLI name is `nazuko`.
- Rust core and UI are separate processes.
- Core license is AGPL-3.0-or-later; `NOTICE` carries SilentStudio attribution.
- Software does not auto-download MaxMind data. Users register at MaxMind,
  download `.mmdb` files, and select a local directory.
- Platform differences are plugins, not separate products.
- Windows capture uses Npcap through a plugin. Npcap remains a separate
  proprietary dependency: user-installed by default, OEM redistribution if
  bundled. macOS and Linux use libpcap/BPF.
- No external browser is launched as the analysis surface. Browser engines run
  inside Nazu&Ko-controlled Browser Host processes.
- Source reuse is license-gated: integrate compatible libraries/engines, not
  whole third-party products with their own UI.

TDD Route: off. This rename/documentation slice is mechanical and verification
is compile/launch/URL inspection. Future behavior work uses the project quality
gates and risk-based tests.

## 2. Product shape

One IDE-like shell:

    top: native menu / titlebar
    left: Explorer
    center: Workbench
    right: Inspector
    bottom: Console / logs / terminal
    very bottom: status/task island (30px, expandable upward)

Modes change the Explorer and Workbench, not the shell:

- Intercept: HTTP/API proxy, request/response, Repeater
- Packets: interfaces, packet list, protocol tree, Hex
- Forensics: Hosts, Sessions, Files, Images, Credentials, DNS
- Browser: pages, network, console, DevTools

The bottom island is the global task surface: ready state, task N/M, progress,
proxy/capture state, GeoIP state. It expands only when there is something to
show.

## 3. Architecture

    Python / PySide6 UI
      ├─ IDE shell + bottom island
      ├─ Browser view host
      └─ control client

    Rust core / daemon
      ├─ capture manager + libpcap / Npcap backend
      ├─ HTTP/1.1, TLS MITM, HTTP/2, WebSocket
      ├─ raw TCP / Minecraft
      ├─ session store + rules + replay
      └─ local MMDB reader

    Platform plugins
      ├─ CaptureBackend-Windows → Npcap
      ├─ CaptureBackend-macOS   → libpcap / BPF
      └─ CaptureBackend-Linux   → libpcap

No UI code parses untrusted packets. No capture backend bypasses the session
store. No browser engine gets direct network access; its egress goes through
the Nazu&Ko network path.

## 4. Phases

### P0 — Identity and legal foundation

- [x] Rename GitHub repository to `Nazu-and-Ko`.
- [x] Rename local repository directory.
- [x] Update remote URL, Cargo repository URL, README branding.
- [x] Replace Apache-2.0 with AGPL-3.0-or-later.
- [x] Add NOTICE with SilentStudio attribution.
- [x] Rename visible Python UI title/about/export names.
- [ ] Internal crate and binary rename (`nettool-*`, `nettoold`) with a
      compatibility period.
- [ ] Retire remaining web-dashboard code in the frozen Qt client.
- [ ] Migrate Python config path to `~/.config/nazuko/` while importing the
      old config on first run.
- [ ] License audit for third-party code and dependencies.

Acceptance: repository URL exact, Cargo metadata license/repository correct,
Python UI launches with Nazu&Ko title, no old visible product name in the
official UI.

### P1 — IDE shell and bottom island

- Replace the current two-dock layout with Explorer / Workbench / Inspector /
  Console.
- Add full-width bottom status island, 30px collapsed, animated expansion.
- Add a task manager with task N/M, progress, cancel and error states.
- Validate with fake sessions before connecting real capture.

Acceptance: no empty center, panels fill and remain user-resizable, island
shows ready/task states, no modal task UI.

### P2 — Capture and session core

- Define the CaptureBackend plugin ABI.
- Windows Npcap backend; macOS libpcap/BPF backend.
- PCAP/PCAPNG import/export.
- Unified session model: source, protocol, request/response, raw packets,
  metadata, rule hits.
- Local MMDB support: directory picker, City/Country/ASN files, validation,
  build date, reload. No network download.

Acceptance: capture can start/stop on both platforms, sessions fill the
Explorer, invalid MMDB never crashes the app, no outbound MMDB request exists.

### P3 — HTTP/API workbench

- HTTP/1.1 proxy and session list.
- Request/response inspector, Raw/Hex/HTTP/JSON.
- Repeater.
- Basic allow/drop rules.
- TLS passthrough; TLS MITM and local CA follow after the plaintext loop.

Acceptance: capture → inspect → edit → replay works end to end; proxy address
and task state are visible in the island.

### P4 — Raw TCP, Minecraft and forensics

- Raw TCP session view.
- Minecraft framing and packet decoding.
- Host aggregation, TCP stream reassembly.
- HTTP file extraction, DNS, credentials, host/OS metadata.
- LAN capture via libpcap/Npcap; no promise of switched-network visibility
  without mirror/SPAN or authorization.

Acceptance: one capture store serves packet, host and artifact views.

### P5 — Browser analysis

- Chromium Browser Host with DevTools/CDP.
- Browser traffic forced through the Nazu&Ko network path.
- Separate profile, sandbox and crash domain.
- Light engine later (WKWebView/WebView2/WebKitGTK).
- Firefox/Gecko only as a dedicated fork and Browser Host; not an external
  browser dependency.

Acceptance: browser pages, network and console are inspectable; engine cannot
bypass the proxy; a browser crash does not take down capture.

## 5. Legal and security gates

- AGPL-3.0-or-later in `LICENSE`; attribution in `NOTICE`.
- Do not link proprietary code into the AGPL core.
- Npcap: Windows-only, proprietary; user install or OEM redistribution.
- MaxMind: code reads local `.mmdb`; data is not redistributed.
- Wireshark/NetworkMiner features are reimplemented or integrated only when
  their licenses allow it; no wholesale UI/product embedding.
- Browser engines keep their own licenses; our integration code remains AGPL.
- Every third-party dependency needs an SPDX/license entry and CI gate.

## 6. Verification

- `cargo metadata --no-deps` shows AGPL and the new repository URL.
- `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings`.
- `python3 -m py_compile main.py`; launch Python UI and verify title/about.
- `git remote -v` uses `https://github.com/SilentCodeTeams/Nazu-and-Ko.git`.
- MMDB checks are offline-only.

## 7. Open decisions

- Exact Npcap distribution model: user install vs OEM.
- Firefox/Gecko fork budget and whether it is a real product requirement.
- MaxMind data license/attribution wording.
- Whether official modules stay AGPL or use a separate service/data license.
