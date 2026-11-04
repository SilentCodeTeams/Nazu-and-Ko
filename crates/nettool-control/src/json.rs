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

//! Small dependency-free JSON codec for the control channel.
//!
//! The control protocol only needs objects, arrays, strings, booleans, null and
//! numbers, so this parser is intentionally small and strict. It is not a
//! general-purpose JSON library.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(BTreeMap<String, Json>),
}

impl Json {
    pub fn parse(input: &str) -> Result<Self, String> {
        parse(input)
    }

    pub fn as_object(&self) -> Option<&BTreeMap<String, Json>> {
        match self {
            Self::Object(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Json]> {
        match self {
            Self::Array(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Self::Number(value) if value.is_finite() && *value >= 0.0 && value.fract() == 0.0 => {
                Some(*value as u64)
            }
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&Json> {
        self.as_object().and_then(|object| object.get(key))
    }

    pub fn to_json_string(&self) -> String {
        let mut out = String::new();
        self.write_json(&mut out);
        out
    }

    fn write_json(&self, out: &mut String) {
        match self {
            Self::Null => out.push_str("null"),
            Self::Bool(value) => out.push_str(if *value { "true" } else { "false" }),
            Self::Number(value) => {
                if value.is_finite() && value.fract() == 0.0 {
                    out.push_str(&format!("{}", *value as i64));
                } else {
                    out.push_str(&format!("{}", value));
                }
            }
            Self::String(value) => write_string(value, out),
            Self::Array(values) => {
                out.push('[');
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    value.write_json(out);
                }
                out.push(']');
            }
            Self::Object(values) => {
                out.push('{');
                for (index, (key, value)) in values.iter().enumerate() {
                    if index > 0 {
                        out.push(',');
                    }
                    write_string(key, out);
                    out.push(':');
                    value.write_json(out);
                }
                out.push('}');
            }
        }
    }
}

fn write_string(value: &str, out: &mut String) {
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            character if (character as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => out.push(character),
        }
    }
    out.push('"');
}

pub fn parse(input: &str) -> Result<Json, String> {
    let mut parser = Parser {
        bytes: input.as_bytes(),
        position: 0,
        depth: 0,
    };
    parser.skip_whitespace();
    let value = parser.parse_value()?;
    parser.skip_whitespace();
    if parser.position != parser.bytes.len() {
        return Err(format!("trailing data at byte {}", parser.position));
    }
    Ok(value)
}

struct Parser<'a> {
    bytes: &'a [u8],
    position: usize,
    depth: usize,
}

impl<'a> Parser<'a> {
    fn parse_value(&mut self) -> Result<Json, String> {
        self.skip_whitespace();
        match self.peek() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'"') => Ok(Json::String(self.parse_string()?)),
            Some(b't') => {
                self.expect_literal("true")?;
                Ok(Json::Bool(true))
            }
            Some(b'f') => {
                self.expect_literal("false")?;
                Ok(Json::Bool(false))
            }
            Some(b'n') => {
                self.expect_literal("null")?;
                Ok(Json::Null)
            }
            Some(b'-') | Some(b'0'..=b'9') => self.parse_number(),
            Some(other) => Err(format!(
                "unexpected byte '{}' at {}",
                other as char, self.position
            )),
            None => Err("unexpected end of input".to_string()),
        }
    }

    fn parse_object(&mut self) -> Result<Json, String> {
        self.enter()?;
        self.expect(b'{')?;
        let mut object = BTreeMap::new();
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.position += 1;
            self.leave()?;
            return Ok(Json::Object(object));
        }
        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.expect(b':')?;
            let value = self.parse_value()?;
            if object.insert(key, value).is_some() {
                return Err("duplicate object key".to_string());
            }
            self.skip_whitespace();
            match self.next() {
                Some(b',') => continue,
                Some(b'}') => break,
                _ => return Err(format!("expected ',' or '}}' at {}", self.position)),
            }
        }
        self.leave()?;
        Ok(Json::Object(object))
    }

    fn parse_array(&mut self) -> Result<Json, String> {
        self.enter()?;
        self.expect(b'[')?;
        let mut values = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.position += 1;
            self.leave()?;
            return Ok(Json::Array(values));
        }
        loop {
            values.push(self.parse_value()?);
            self.skip_whitespace();
            match self.next() {
                Some(b',') => continue,
                Some(b']') => break,
                _ => return Err(format!("expected ',' or ']' at {}", self.position)),
            }
        }
        self.leave()?;
        Ok(Json::Array(values))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect(b'"')?;
        let mut out = String::new();
        let mut segment_start = self.position;
        loop {
            let byte = self
                .next()
                .ok_or_else(|| "unterminated string".to_string())?;
            match byte {
                b'"' => {
                    let segment =
                        std::str::from_utf8(&self.bytes[segment_start..self.position - 1])
                            .map_err(|_| "invalid UTF-8 in string".to_string())?;
                    out.push_str(segment);
                    return Ok(out);
                }
                b'\\' => {
                    let segment =
                        std::str::from_utf8(&self.bytes[segment_start..self.position - 1])
                            .map_err(|_| "invalid UTF-8 in string".to_string())?;
                    out.push_str(segment);
                    let escaped = self
                        .next()
                        .ok_or_else(|| "unterminated escape".to_string())?;
                    match escaped {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let code = self.parse_hex4()?;
                            let character = char::from_u32(code)
                                .ok_or_else(|| "invalid unicode escape".to_string())?;
                            out.push(character);
                        }
                        other => return Err(format!("invalid escape '\\{}'", other as char)),
                    }
                    segment_start = self.position;
                }
                0x00..=0x1f => {
                    return Err(format!("control byte at {}", self.position));
                }
                _ => {}
            }
        }
    }

    fn parse_hex4(&mut self) -> Result<u32, String> {
        let mut value = 0u32;
        for _ in 0..4 {
            let byte = self
                .next()
                .ok_or_else(|| "short unicode escape".to_string())?;
            let digit = match byte {
                b'0'..=b'9' => (byte - b'0') as u32,
                b'a'..=b'f' => (byte - b'a' + 10) as u32,
                b'A'..=b'F' => (byte - b'A' + 10) as u32,
                _ => return Err("invalid unicode escape".to_string()),
            };
            value = (value << 4) | digit;
        }
        Ok(value)
    }

    fn parse_number(&mut self) -> Result<Json, String> {
        let start = self.position;
        if self.peek() == Some(b'-') {
            self.position += 1;
        }
        let mut digits = 0;
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.position += 1;
            digits += 1;
        }
        if digits == 0 {
            return Err(format!("invalid number at {}", start));
        }
        if self.peek() == Some(b'.') {
            self.position += 1;
            let mut fraction = 0;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.position += 1;
                fraction += 1;
            }
            if fraction == 0 {
                return Err(format!("invalid fraction at {}", start));
            }
        }
        if matches!(self.peek(), Some(b'e') | Some(b'E')) {
            self.position += 1;
            if matches!(self.peek(), Some(b'+') | Some(b'-')) {
                self.position += 1;
            }
            let mut exponent = 0;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.position += 1;
                exponent += 1;
            }
            if exponent == 0 {
                return Err(format!("invalid exponent at {}", start));
            }
        }
        let text = std::str::from_utf8(&self.bytes[start..self.position])
            .map_err(|_| "invalid number".to_string())?;
        let value = text
            .parse::<f64>()
            .map_err(|_| format!("invalid number '{text}'"))?;
        if !value.is_finite() {
            return Err("number is not finite".to_string());
        }
        Ok(Json::Number(value))
    }

    fn expect_literal(&mut self, literal: &str) -> Result<(), String> {
        let bytes = literal.as_bytes();
        if self.bytes.get(self.position..self.position + bytes.len()) == Some(bytes) {
            self.position += bytes.len();
            Ok(())
        } else {
            Err(format!("expected '{literal}' at {}", self.position))
        }
    }

    fn enter(&mut self) -> Result<(), String> {
        self.depth += 1;
        if self.depth > 64 {
            return Err("JSON nesting too deep".to_string());
        }
        Ok(())
    }

    fn leave(&mut self) -> Result<(), String> {
        self.depth = self.depth.saturating_sub(1);
        Ok(())
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.position += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    fn next(&mut self) -> Option<u8> {
        let value = self.peek()?;
        self.position += 1;
        Some(value)
    }

    fn expect(&mut self, expected: u8) -> Result<(), String> {
        match self.next() {
            Some(actual) if actual == expected => Ok(()),
            _ => Err(format!(
                "expected '{}' at {}",
                expected as char, self.position
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_object_and_array() {
        let value = parse(r#"{"a":1,"b":["x",true,null],"c":{"d":2}}"#).unwrap();
        assert_eq!(value.get("a").and_then(Json::as_u64), Some(1));
        assert_eq!(
            value.get("b").and_then(Json::as_array).map(|v| v.len()),
            Some(3)
        );
        assert_eq!(
            value
                .get("c")
                .and_then(|v| v.get("d"))
                .and_then(Json::as_u64),
            Some(2)
        );
    }

    #[test]
    fn parses_escaped_strings() {
        let value = parse(r#"{"text":"a\nb\t\"c\""}"#).unwrap();
        assert_eq!(
            value.get("text").and_then(Json::as_str),
            Some("a\nb\t\"c\"")
        );
    }

    #[test]
    fn serializes_and_reparses() {
        let value = parse(r#"{"a":[1,true,"x"]}"#).unwrap();
        let encoded = value.to_json_string();
        let decoded = parse(&encoded).unwrap();
        assert_eq!(value, decoded);
    }

    #[test]
    fn rejects_invalid_json() {
        for input in ["", "{", "{\"a\":}", "{\"a\":1} trailing", "[1,]"] {
            assert!(parse(input).is_err(), "expected invalid: {input}");
        }
    }
}
