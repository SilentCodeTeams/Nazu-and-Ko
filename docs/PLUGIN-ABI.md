<!--
本文件由 SilentStudio 监管与负责。
本文件属于 SilentCodeTeams 产品 Nazu&Ko 的源码部分
本项目由 SilentCodeTeams 对法律、版权事宜及相关咨询承担责任。
版权所有 © SilentStudio。
以 AGPL-3.0-or-later 授权。
This document is supervised and responsible by SilentStudio.
This file is part of the source code of the SilentCodeTeams product Nazu&Ko
SilentCodeTeams is responsible for legal and copyright issues and related consultation in this project.
Copyright © SilentStudio.
Licensed as AGPL-3.0-or-later.
-->

# Plugin ABI (design, not implemented yet)

Goal: official and third-party plugins as .dylib/.dll/.so on macOS, Windows
and Linux, without exposing the Rust ABI.

## Rules

- Only C ABI types cross the boundary. No String, Vec, trait objects or Rust
  enums directly.
- The host calls into the plugin through a versioned vtable; the plugin does
  not link against the host.
- Every entry point is wrapped in catch_unwind on the Rust side.
- ABI version mismatch is a hard reject.
- Plugin manifest carries name, version, abi_version, os, arch and
  capabilities. Official plugins are Ed25519-signed; the host verifies before
  loading.
- Untrusted plugins run out of process in a later phase.

## Sketch

    // plugin exports
    #[no_mangle]
    pub extern "C" fn nettool_plugin_abi() -> u32 { 1 }

    #[no_mangle]
    pub extern "C" fn nettool_plugin_init(host: *const HostApi) -> i32;

    #[no_mangle]
    pub extern "C" fn nettool_plugin_shutdown();

    // host-provided table (subset)
    #[repr(C)]
    pub struct HostApi {
        pub abi_version: u32,
        pub log: extern "C" fn(level: u32, msg: *const c_char, len: usize),
        pub emit_packet: extern "C" fn(event: *const PacketEvent),
    }

## Platform notes

- macOS: hardened runtime, codesign, notarize; load plugins by absolute path.
- Windows: SetDefaultDllDirectories and absolute paths to avoid DLL planting;
  ship the MSVC runtime.
- Linux: RPATH or $ORIGIN; prefer a private plugin directory per user.

## Packaging

Each plugin is built per target:

    macOS:   libnettool_plugin_foo.dylib (arm64 + x86_64 universal)
    Windows: nettool_plugin_foo.dll (x64, arm64)
    Linux:   libnettool_plugin_foo.so (x86_64, optional aarch64)

Manifest example:

    {
      "name": "mc-dissector",
      "version": "0.1.0",
      "abi": 1,
      "os": "macos",
      "arch": "arm64",
      "capabilities": ["dissector"],
      "signature": "hex-ed25519"
    }
