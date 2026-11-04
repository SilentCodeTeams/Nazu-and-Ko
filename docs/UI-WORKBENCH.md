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

# Workbench and layout direction

Status: design recorded, implementation paused until the next session.

## Intent

The application should feel like an IDE, not a dashboard with docked tables. The
central Workbench is the main working surface: it can edit code, hold prompts,
and inspect captured data. The shell keeps movable Qt docks, but the default
layout must be dense and filled, never a large empty centre.

## Target layout

    ┌─────────────────────────────────────────────────────────────────────┐
    │ menu / native titlebar                                              │
    ├──────────┬──────────────────────┬───────────────────────┬────────────┤
    │ file     │ traffic / sessions   │ Workbench / code      │ right      │
    │ 1/4 left │ 3/4 left             │ editor, no tabs       │ function   │
    ├──────────┴──────────────────────┴───────────────────────┴────────────┤
    │ Console / terminal, full width                                      │
    ├─────────────────────────────────────────────────────────────────────┤
    │ status island, full width                                           │
    └─────────────────────────────────────────────────────────────────────┘

Four columns across the full width, left to right:

1. File/project explorer: 1/4 of the left block
2. Traffic/session functional area: 3/4 of the left block
3. Workbench: the code/editor column between traffic and the right function area
4. Right function area: Inspector and tool entry points

The left block is still split 1/4 + 3/4. Both left panels remain movable and
resizable Qt docks. The Workbench is a separate centre column, not a row below
the columns.

Bottom: Console/terminal spans the full width.

Very bottom: existing status island, unchanged.

## Workbench

Remove the Request Editor / Response Editor tab bar. They were inherited from the
old layout and take visual priority away from the Workbench.

The third column becomes one Workbench surface:

- code mode
- prompt mode
- data mode

The packet table feeds the Workbench in data mode automatically. A selected
packet chooses the data view; there is no user-facing request/response tab.

Modes are switched by:

- Ctrl+1: code
- Ctrl+2: prompt
- Ctrl+3: data
- compact text buttons at the top of the Workbench

The buttons should be subtle text segments, not large filled Fluent pills. Target
height 22-24px, muted text, active state shown by a thin underline or a very
light background. They must not dominate the editor.

Data mode keeps Raw / Hex / JSON and the existing editor actions.

## Editor

Requirements:

- line numbers
- syntax highlighting
- bracket matching, including unmatched bracket warning
- minimap/overview on the right side of the Workbench
- code, prompt and data buffers are independent
- soft editor padding and selection colours; avoid the current hard boxed look

Highlighting:

- Pygments 2.21.0 is available on this machine
- use Pygments lexers for broad language coverage, with automatic lexer guess
- "all VSCode languages" is not literally achievable; Pygments provides a broad
  lexer set and a plain-text fallback
- keep a lightweight fallback highlighter if Pygments is unavailable
- do not block typing while a lexer is guessed; debounce expensive work

Line numbers:

- custom line-number gutter tied to QPlainTextEdit block count and viewport
- match the current light/dark theme

Minimap:

- custom QWidget on the right edge of the Workbench editor
- draw one vertical line per text block scaled to height
- show the current viewport rectangle
- click/drag to scroll
- hide in Hex view
- must remain responsive for long documents

## Interaction rules

- Packet selection updates Inspector and data-mode Workbench
- Double-click switches Workbench to data mode and loads the packet
- Mode state is per Workbench, not per old request/response tab
- Console keeps proxy/client/packet logs
- Status island keeps ready/task/progress state
- Do not reintroduce a request/response tab strip

## Current implementation state

Already present in the working Python UI:

- four-zone IDE shell
- Explorer, Inspector, Console docks
- status island
- mode buttons and Ctrl+1/2/3 in PacketEditor
- basic bracket matcher and keyword highlighter
- data mode Raw/Hex/JSON

Not yet done:

- left region split into 1/4 file explorer + 3/4 traffic area
- remove central request/response tabs
- Pygments-backed language highlighter
- line-number gutter
- minimap
- final subtle mode-button styling
- full light/dark verification

## Acceptance

- no empty centre at default 1400x900
- left explorer is about 1/4 of the left block; traffic area about 3/4
- Workbench is the third column, between traffic and the right function area
- Workbench has no request/response tab strip
- Ctrl+1/2/3 and buttons both switch modes
- mode controls stay visually secondary
- line numbers and minimap visible in code/prompt modes
- bracket pair highlights on cursor move; unmatched bracket is visibly marked
- Pygments language mode detects common code; plain fallback never crashes
- data mode still opens captured packets and supports Raw/Hex/JSON
- all docks remain movable and resizable

## Next session

1. Split the left docks.
2. Remove the central request/response tabs and keep one Workbench editor.
3. Integrate Pygments with theme-aware formats.
4. Add line numbers.
5. Add minimap.
6. Soften the shell and mode controls.
7. Test light and dark at 1400x900 and a narrower window.
