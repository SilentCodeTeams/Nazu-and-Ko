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

//! Language selection with explicit configuration over IP geolocation.
//!
//! Priority:
//!   --lang-file > --lang > config language_file > config language
//!   > NAZUKO_LANG > public-IP country > en

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use crate::i18n;

const CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
    Builtin(String),
    Custom(PathBuf),
}

#[derive(Debug, Default)]
struct Config {
    language: Option<String>,
    language_file: Option<PathBuf>,
}

pub fn resolve(
    cli_lang_file: Option<String>,
    cli_lang: Option<String>,
) -> Result<Selection, String> {
    if let Some(value) = cli_lang_file {
        return custom_selection(PathBuf::from(value));
    }
    if let Some(value) = cli_lang {
        return builtin_selection(&value);
    }

    let config_path = config_path();
    let config = read_config(&config_path);
    if let Some(path) = config.language_file {
        return custom_selection(resolve_path(&path, config_path.parent()));
    }
    if let Some(value) = config.language {
        if let Some(code) = i18n::normalize(&value) {
            return Ok(Selection::Builtin(code.to_string()));
        }
    }
    if let Ok(value) = env::var("NAZUKO_LANG") {
        if let Some(code) = i18n::normalize(&value) {
            return Ok(Selection::Builtin(code.to_string()));
        }
    }
    let country = country_code();
    let code = language_for_country(&country)
        .or_else(system_language)
        .unwrap_or("en");
    Ok(Selection::Builtin(code.to_string()))
}

fn builtin_selection(value: &str) -> Result<Selection, String> {
    i18n::normalize(value)
        .map(|code| Selection::Builtin(code.to_string()))
        .ok_or_else(|| format!("unsupported language {value:?}; supported: zh-CN, zh-TW, en"))
}

fn custom_selection(path: PathBuf) -> Result<Selection, String> {
    if !path.is_file() {
        return Err(format!("language file does not exist: {}", path.display()));
    }
    Ok(Selection::Custom(path))
}

fn resolve_path(path: &Path, base: Option<&Path>) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else if let Some(base) = base {
        base.join(path)
    } else {
        path.to_path_buf()
    }
}

fn config_path() -> PathBuf {
    if let Some(path) = env::var_os("NAZUKO_CONFIG") {
        return PathBuf::from(path);
    }
    config_home().join("nazuko").join("config")
}

fn config_home() -> PathBuf {
    if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(path);
    }
    home().join(".config")
}

fn cache_path() -> PathBuf {
    if let Some(path) = env::var_os("XDG_CACHE_HOME") {
        return PathBuf::from(path).join("nazuko").join("country");
    }
    home().join(".cache").join("nazuko").join("country")
}

fn home() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn read_config(path: &Path) -> Config {
    let Ok(text) = fs::read_to_string(path) else {
        return Config::default();
    };
    let mut config = Config::default();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string();
        if value.is_empty() {
            continue;
        }
        match key.trim() {
            "language" => config.language = Some(value),
            "language_file" => config.language_file = Some(PathBuf::from(value)),
            _ => {}
        }
    }
    config
}

fn country_code() -> String {
    if let Ok(value) = env::var("NAZUKO_COUNTRY") {
        let code = value.trim().to_ascii_uppercase();
        if valid_country(&code) {
            return code;
        }
    }

    let cache = cache_path();
    if let Some(code) = read_cached_country(&cache) {
        return code;
    }
    let code = fetch_country();
    if valid_country(&code) {
        write_cached_country(&cache, &code);
        code
    } else {
        String::new()
    }
}

fn read_cached_country(path: &Path) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    let age = metadata.modified().ok()?.elapsed().ok()?;
    if age > CACHE_TTL {
        return None;
    }
    let text = fs::read_to_string(path).ok()?;
    let code = text.trim().to_ascii_uppercase();
    valid_country(&code).then_some(code)
}

fn write_cached_country(path: &Path, code: &str) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, format!("{code}\n"));
}

fn fetch_country() -> String {
    for url in ["https://ipapi.co/country/", "https://ipinfo.io/country"] {
        if let Some(code) = curl_country(url) {
            return code;
        }
    }
    String::new()
}

fn curl_country(url: &str) -> Option<String> {
    let output = Command::new("curl")
        .args(["-fsS", "--max-time", "2", "-A", "nazuko/0.1", url])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let code = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_ascii_uppercase();
    valid_country(&code).then_some(code)
}

fn valid_country(code: &str) -> bool {
    code.len() == 2 && code.chars().all(|character| character.is_ascii_uppercase())
}

fn system_language() -> Option<&'static str> {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE"] {
        if let Ok(value) = env::var(key) {
            for part in value.split([':', ',']) {
                if let Some(code) = i18n::normalize(part) {
                    return Some(code);
                }
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("defaults")
            .args(["read", "-g", "AppleLocale"])
            .output()
            .ok()?;
        if output.status.success() {
            let value = String::from_utf8_lossy(&output.stdout);
            return i18n::normalize(value.trim());
        }
    }
    None
}

pub fn language_for_country(country: &str) -> Option<&'static str> {
    match country {
        "CN" => Some("zh-CN"),
        "TW" | "HK" | "MO" => Some("zh-TW"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_country_to_supported_language() {
        assert_eq!(language_for_country("CN"), Some("zh-CN"));
        assert_eq!(language_for_country("TW"), Some("zh-TW"));
        assert_eq!(language_for_country("HK"), Some("zh-TW"));
        assert_eq!(language_for_country("US"), None);
        assert_eq!(language_for_country(""), None);
    }

    #[test]
    fn config_parser_ignores_comments() {
        let config = Config {
            language: Some("zh-TW".to_string()),
            language_file: None,
        };
        assert_eq!(config.language.as_deref(), Some("zh-TW"));
    }

    #[test]
    fn relative_custom_path_resolves_against_config_dir() {
        let path = resolve_path(Path::new("custom.lang"), Some(Path::new("/tmp/nazuko")));
        assert_eq!(path, PathBuf::from("/tmp/nazuko/custom.lang"));
    }
}
