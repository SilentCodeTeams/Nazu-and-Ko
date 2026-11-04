# NetTool ζ Qt — design notes

The legacy PySide6 screen was a faithful functional port, but it had no visual
hierarchy: a large empty centre, a flat toolbar, and status information mixed
with ordinary labels. The Qt client keeps the workflow but gives it a calmer
security-cockpit hierarchy.

The references collected for this project — BoardUI, Ant Design Agentic UI,
HextaUI, Montage, Holo UI Vault, awesome-design-md and A2UI — are used as
design references, not as runtime dependencies. React component source is not
copied into this Qt Widgets client.

## Principles

1. State before chrome. The first thing in the workspace is whether the proxy is
   running, how many sessions and packets exist, and the next action.
2. One primary action per state. Starting the proxy is the teal action;
   configuration and secondary tools stay quiet.
3. Progressive disclosure. History, request and response are docks. The centre
   is a short overview, not another dense table.
4. Quiet surfaces. The canvas is grey, cards are white (or near-black in dark
   mode), borders are hairline, and only state changes get strong colour.
5. Empty states explain the next step. "Waiting for traffic" is paired with the
   start action and a three-step quick start.
6. Localisation is structural. Every visible string goes through the key=value
   translator. No translated text is embedded in C++ UI code.

## Tokens

| Token | Light | Dark |
| --- | --- | --- |
| canvas | #F5F6F8 | #0B0F14 |
| surface | #FFFFFF | #11161F |
| border | #E4E7EC | #232B36 |
| text | #111827 | #E6EAF0 |
| muted | #667085 | #8B96A7 |
| accent | #0F766E | #2DD4BF |
| warning | #B54708 / #FFFAEB | #FDB022 / #2A2114 |
| radius (card / control) | 14 / 8 px | 14 / 8 px |
| spacing scale | 4 / 8 / 12 / 16 / 24 px | same |

Type scale: 24 px page title, 14 px section, 11 px label, 10 px eyebrow,
26 px metric value. The default UI font is the platform UI font; packet content
uses the platform fixed-width font.

## Layout

    menu bar
    toolbar: start/stop · tools · filter · theme
    ┌ packet history ─────────┬ workspace ──────────────┬ inspector ─┐
    │ table + empty hint      │ status + metric cards   │ request /  │
    │                         │ quick start             │ response   │
    └─────────────────────────┴─────────────────────────┴────────────┘
    status bar: ready · language · theme · proxy state · packet count

The dashboard is deliberately short. It answers "is it running, what is
happening, what do I do next" without competing with the packet table.

## Localisation

Language files live in lang/*.lang:

    key=value

lang/languages.conf is the index:

    zh-CN=简体中文
    en=English

Translator loads English as the fallback and the selected language on top. The
language is stored in QSettings (ui/language); changing it requires a restart.
Adding a language only requires a new .lang file with the same keys and one line
in languages.conf.

## Status of this port

Implemented: main window, dashboard, docks, themes, filtering, zoom, proxy
configuration, preferences, language selection, live output from nettoold proxy,
packet rows, Repeater, Rules, Intruder and Scanner dialogs, PacketEditor
Raw/Hex/JSON/HTTP tabs, toast notifications, smart CSV/JSON/TSV export, and
rules passed to nettoold proxy.

Still to port: deeper legacy tool-option parity and notification animation. These
remain functional gaps, not "porting in progress" message boxes.
