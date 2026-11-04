# Nazu&Ko Qt client

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

Implemented:

- Main window, dashboard, docks and status bar
- Light/dark/system themes, filtering and zoom
- Key=value localisation with Chinese and English UI strings
- Proxy configuration, preferences and language selection
- Start/stop of the Rust `nettoold proxy` process, with captured packet rows
- Repeater, Rules, Intruder and Scanner dialogs; JSON rule import/export
- Repeater redirect/timeout options and a 20-entry request history
- Intruder and Scanner concurrency, delay and stop controls
- Intruder raw HTTP-template mode with a §payload§ marker
- Notification enable/duration/placement/animation settings, honoured by the toast centre
- Proxy auto-start setting
- PacketEditor Raw/Hex/JSON/HTTP tabs with JSON/HTTP highlighting
- Loopback web dashboard at / and /api/status
- Toast notification centre
- Smart export in CSV, JSON and TSV, all rows or visible rows
- Rules saved in the Rules dialog are passed to `nettoold proxy --rule`

Known gaps from full legacy parity:

- Deeper legacy option parity and richer web dashboard history
- Additional translations beyond Chinese and English

The tool dialogs are functional Qt ports, not placeholder message boxes. They do
not yet have every legacy option (for example, Intruder concurrency controls),
but every menu entry opens an actual window.

## Language files

UI strings live in `lang/*.lang` as `key=value`. The index is
`lang/languages.conf`:

    zh-CN=简体中文
    en=English

English is always the fallback. To add a language, copy `lang/en.lang`, translate
the values, add one line to `languages.conf`, and rebuild so CMake copies the
language directory next to the executable.

Run with a specific language:

    ./build/qt/nettool-qt --lang zh-CN
