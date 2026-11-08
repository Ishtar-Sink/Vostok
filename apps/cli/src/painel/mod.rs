//! Painel local (`vostok painel`): servidor HTTP só em `127.0.0.1`, que serve o app de `apps/painel`,
//! a leitura dos arquivos de `vostok/` e o aviso ao vivo das mudanças.

pub mod live;
pub mod read;
pub mod server;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::harness::Harness;

/// Resposta HTTP já montada, sem depender do servidor.
#[derive(Debug, PartialEq, Eq)]
pub struct Reply {
    pub status: u16,
    pub content_type: &'static str,
    pub body: Vec<u8>,
    pub headers: Vec<(&'static str, String)>,
}

/// O harness servido pelo painel.
pub struct Painel {
    pub project: PathBuf,
    pub root: PathBuf,
    pub work: PathBuf,
}

impl Painel {
    pub fn new(harness: &Harness) -> Result<Self> {
        let root = fs::canonicalize(&harness.root)
            .with_context(|| format!("harness inacessível em {}", harness.root.display()))?;
        let project = root
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| root.clone());
        Ok(Self {
            work: root.join("work"),
            project,
            root,
        })
    }
}

pub fn text(status: u16, message: &str) -> Reply {
    Reply {
        status,
        content_type: "text/plain; charset=utf-8",
        body: message.as_bytes().to_vec(),
        headers: Vec::new(),
    }
}

pub fn not_found() -> Reply {
    text(404, "nao encontrado")
}

pub fn json(body: String) -> Reply {
    Reply {
        status: 200,
        content_type: "application/json; charset=utf-8",
        body: body.into_bytes(),
        headers: Vec::new(),
    }
}

/// Texto como string JSON, com as aspas e os escapes.
pub fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if (control as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", control as u32))
            }
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

/// Valor decodificado de uma chave da consulta; vazio quando ela falta.
pub fn query_value(query: &str, key: &str) -> String {
    query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find(|(name, _)| *name == key)
        .map(|(_, value)| percent_decode(value))
        .unwrap_or_default()
}

pub fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let decoded = bytes
            .get(index + 1..index + 3)
            .filter(|_| bytes[index] == b'%')
            .and_then(|hex| std::str::from_utf8(hex).ok())
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match (decoded, bytes[index]) {
            (Some(byte), _) => {
                out.push(byte);
                index += 3;
                continue;
            }
            (None, b'+') => out.push(b' '),
            (None, byte) => out.push(byte),
        }
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Um segmento de caminho sem separadores nem `..`: letras, dígitos, `-` e `_`.
pub fn safe_segment(value: &str) -> Result<&str, Reply> {
    let safe = !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if safe {
        Ok(value)
    } else {
        Err(not_found())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_segment_accepts_ids() {
        assert_eq!(safe_segment("SPEC-029").unwrap(), "SPEC-029");
        assert_eq!(safe_segment("task_02").unwrap(), "task_02");
    }

    #[test]
    fn safe_segment_refuses_traversal_separators_and_empty() {
        for value in ["..", "../x", "a/b", "a\\b", "", ".", "a b", "%2e%2e"] {
            assert_eq!(safe_segment(value).unwrap_err().status, 404, "{value}");
        }
    }

    #[test]
    fn query_value_decodes_and_defaults_to_empty() {
        let query = "spec=SPEC-029&caminho=work%2Fx.md&texto=um+dois";
        assert_eq!(query_value(query, "spec"), "SPEC-029");
        assert_eq!(query_value(query, "caminho"), "work/x.md");
        assert_eq!(query_value(query, "texto"), "um dois");
        assert_eq!(query_value(query, "falta"), "");
        assert_eq!(query_value("", "spec"), "");
    }

    #[test]
    fn percent_decode_keeps_invalid_or_truncated_escapes() {
        assert_eq!(percent_decode("%zz"), "%zz");
        assert_eq!(percent_decode("a%2"), "a%2");
        assert_eq!(percent_decode("%"), "%");
        assert_eq!(percent_decode("%C3%A7"), "ç");
    }

    #[test]
    fn json_string_escapes_quotes_and_controls() {
        assert_eq!(json_string("a\"b\\c\n\u{1}"), "\"a\\\"b\\\\c\\n\\u0001\"");
    }
}
