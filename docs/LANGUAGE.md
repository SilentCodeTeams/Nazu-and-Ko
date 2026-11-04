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

# CLI language

## Priority

```text
--lang-file
> --lang
> config language_file
> config language
> NAZUKO_LANG
> public IP country
> system language
> en
```

A persisted user setting always wins over IP geolocation. A custom language file
wins over everything, including a conflicting `--lang`.

## Built-in languages

- `zh-CN` — Simplified Chinese
- `zh-TW` — Traditional Chinese
- `en` — English

Built-in catalogs are embedded in the binary, so `cargo install` does not need a
separate data directory.

## Config

Default path:

```text
~/.config/nazuko/config
```

Override with `NAZUKO_CONFIG`. Keys:

```ini
language=zh-CN
language_file=/absolute/path/to/custom.lang
```

`language_file` wins over `language`.

## Custom language file

```bash
nazuko --lang-file /path/to/custom.lang help
```

- UTF-8, `key=value`, `#` comments
- Maximum 256 KB
- Missing keys fall back to English
- The file may use a language code not in the built-in list
- Invalid or missing files are reported as errors

Example:

```ini
cli.help=My custom help for {version}
cli.error.prefix=Custom error: 
```

## Public IP fallback

Only used when no explicit setting or config exists.

- `CN` → `zh-CN`
- `TW`, `HK`, `MO` → `zh-TW`
- every other country → system language
- lookup failure → system language

The lookup calls `https://ipapi.co/country/` through the system `curl` with a
2-second timeout and a `nazuko/0.1` User-Agent; `https://ipinfo.io/country` is
the fallback. Only the two-letter country code is cached for 24 hours; the full
IP is not logged. VPNs and corporate egress can make the country wrong, so the
user setting always has final authority.

If the country is unsupported or the lookup fails, the resolver uses the OS
locale (`LC_ALL`, `LC_MESSAGES`, `LANG`, `LANGUAGE`, then macOS `AppleLocale`).
If that is also unsupported, English is used.

## MCP

MCP is not localized:

- protocol messages, tool descriptions and errors are English
- errors are short and stable to save tokens
- tool execution errors use `isError: true`
- parameter errors use JSON-RPC `-32602`
- CLI language settings never change MCP output
