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

# Qt parity checklist

Goal scope: the Qt 6 client feature set listed in the goal, with every UI string
in lang/*.lang key=value files.

## Delivered

- [x] PacketEditor tabs: Raw, Hex, JSON, HTTP
- [x] JSON and HTTP syntax highlighting, pretty-print, copy
- [x] Toast notification centre with enable, duration, placement and animation
- [x] Smart export: CSV, JSON, TSV; all or visible rows
- [x] Repeater: redirect/timeout options and request history
- [x] Rules: add/remove, JSON import/export, expressions passed to nazuko proxy
- [x] Intruder: URL/FUZZ and raw HTTP template modes, concurrency, delay, stop
- [x] Scanner: paths, concurrency, delay, stop
- [x] Settings: theme, font, language, proxy, web, notifications, auto-start
- [x] Loopback web dashboard: / and /api/status
- [x] zh-CN and en language files, 230 keys, key parity, no missing code keys
- [x] No "porting in progress" placeholders

## Evidence

- cmake build succeeds
- QT_QPA_PLATFORM=offscreen ./build/qt/nettool-qt --dialog-smoke exits 0
- curl http://127.0.0.1:2679/ returns HTTP 200
- /api/status returns valid JSON
- client/qt/lang key counts match; used keys are present
- grep finds no porting placeholder strings

## Known non-goal differences

These remain outside this goal round and should be separate work:

- Proxy mode semantics from the legacy app are not enforced by the Rust proxy.
- PacketEditor currently receives the proxy summary line, not full packet bytes.
- Web dashboard history is a recent snapshot rather than a persisted timeline.
