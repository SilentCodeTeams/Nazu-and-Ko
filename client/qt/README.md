# NetTool ζ Qt client

A Qt 6 Widgets port of the legacy PySide6 application. The target is a 1:1
port, done in slices so each slice can be built and inspected.

## Build

Qt 6.11.2 is installed at `/Users/apple/Qt/6.11.2/macos`.

    cmake -S client/qt -B build/qt \
      -DCMAKE_PREFIX_PATH=/Users/apple/Qt/6.11.2/macos
    cmake --build build/qt -j 4

Run:

    ./build/qt/nettool-qt

Headless construction check:

    QT_QPA_PLATFORM=offscreen ./build/qt/nettool-qt --smoke-test

Render a screenshot without opening a window:

    QT_QPA_PLATFORM=offscreen ./build/qt/nettool-qt \
      --screenshot /tmp/nettool-qt.png

## Port status

Ported in this slice:

- Main window, title, geometry and Fusion font
- Menus: 文件 / 代理 / 工具 / 视图 / 设置 / 帮助
- Toolbar: start/stop, Repeater/Intruder/Scanner, clear, filter, theme
- Docks: packet history table, request/response editors
- Status bar: ready, theme, proxy, packet count
- Light/dark/system styles and zoom shortcuts

Pending:

- Packet table live data and proxy backend wiring
- Packet editor tabs, hex/JSON/HTTP highlighting
- ProxyConfigDialog, SettingsDialog
- RepeaterManager, IntruderDialog, ScannerDialog
- SmartExportDialog, RuleManagerDialog
- Web dashboard and notifications

The start/stop actions currently report that the daemon proxy-session API is not
wired yet; they do not fake a running proxy.
