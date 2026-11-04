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

//! Dependency-free CLI message catalog.
//!
//! Built-in catalogs are embedded so an installed binary works without a data
//! directory. A custom key=value file can override the catalog; missing keys
//! fall back to English.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

const ZH_CN: &str = include_str!("../lang/cli.zh-CN.lang");
const ZH_TW: &str = include_str!("../lang/cli.zh-TW.lang");
const EN: &str = include_str!("../lang/cli.en.lang");
const MAX_CUSTOM_BYTES: u64 = 256 * 1024;

struct Catalog {
    entries: HashMap<String, String>,
    fallback: HashMap<String, String>,
}

static CATALOG: OnceLock<Catalog> = OnceLock::new();

pub fn init_code(requested: &str) {
    let code = normalize(requested).unwrap_or("en");
    let embedded = match code {
        "zh-CN" => ZH_CN,
        "zh-TW" => ZH_TW,
        _ => EN,
    };
    let _ = CATALOG.set(Catalog {
        entries: parse(embedded),
        fallback: parse(EN),
    });
}

pub fn init_custom(path: &Path) -> Result<(), String> {
    let metadata = fs::metadata(path)
        .map_err(|error| format!("cannot read language file {:?}: {error}", path))?;
    if metadata.len() > MAX_CUSTOM_BYTES {
        return Err(format!(
            "language file {:?} is too large ({} bytes, maximum {})",
            path,
            metadata.len(),
            MAX_CUSTOM_BYTES
        ));
    }
    let text = fs::read_to_string(path)
        .map_err(|error| format!("cannot read language file {:?}: {error}", path))?;
    let entries = parse(&text);
    if entries.is_empty() {
        return Err(format!(
            "language file {:?} contains no key=value entries",
            path
        ));
    }
    if CATALOG
        .set(Catalog {
            entries,
            fallback: parse(EN),
        })
        .is_err()
    {
        return Err("language catalog already initialized".to_string());
    }
    Ok(())
}

pub fn t(key: &str) -> String {
    let Some(catalog) = CATALOG.get() else {
        return key.to_string();
    };
    catalog
        .entries
        .get(key)
        .or_else(|| catalog.fallback.get(key))
        .cloned()
        .unwrap_or_else(|| key.to_string())
}

pub fn tr_args(key: &str, args: &[(&str, &str)]) -> String {
    let mut text = t(key);
    for (name, value) in args {
        text = text.replace(&format!("{{{name}}}"), value);
    }
    text
}

pub fn normalize(input: &str) -> Option<&'static str> {
    let normalized = input
        .trim()
        .split('.')
        .next()
        .unwrap_or(input)
        .replace('_', "-")
        .to_ascii_lowercase();
    match normalized.as_str() {
        "zh" | "zh-cn" | "zh-hans" | "zh-hans-cn" => Some("zh-CN"),
        "zh-tw" | "zh-hk" | "zh-hant" | "zh-hant-tw" => Some("zh-TW"),
        "en" | "en-us" | "en-gb" => Some("en"),
        _ => None,
    }
}

fn parse(text: &str) -> HashMap<String, String> {
    let mut entries = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        entries.insert(key.trim().to_string(), unescape(value.trim()));
    }
    entries
}

fn unescape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(character) = chars.next() {
        if character != '\\' {
            out.push(character);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_supported_codes() {
        assert_eq!(normalize("zh_CN.UTF-8"), Some("zh-CN"));
        assert_eq!(normalize("zh-TW"), Some("zh-TW"));
        assert_eq!(normalize("en_US"), Some("en"));
        assert_eq!(normalize("fr"), None);
    }

    #[test]
    fn parses_comments_and_escapes() {
        let parsed = parse("# comment\nkey=hello\\nworld\n");
        assert_eq!(parsed.get("key").map(String::as_str), Some("hello\nworld"));
    }
}
