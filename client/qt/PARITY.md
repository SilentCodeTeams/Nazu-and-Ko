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
