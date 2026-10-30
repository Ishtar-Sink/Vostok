//! Perfis de execução e apelidos de reserva por ferramenta (SPEC-014, RF-18 a RF-20).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{bail, Result};

use crate::harness::Harness;
use crate::support::config_get;

pub const SESSION_MODEL: &str = "modelo da sessão";
pub const TOOLS: [&str; 5] = ["claude-code", "codex", "cursor", "gemini-cli", "copilot"];
pub const PROFILES: [&str; 3] = ["mecanica", "padrao", "julgamento"];
const MODELS_FILE: &str = "config/models.yml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileModel {
    pub profile: String,
    pub description: String,
    pub tool: Option<String>,
    pub model: String,
}

/// Achata o YAML de recuo em `caminho.com.pontos` -> valor, ignorando valores vazios.
fn flatten(content: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let mut stack: Vec<(usize, String)> = Vec::new();
    for raw in content.lines() {
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = raw.len() - raw.trim_start().len();
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        while stack.last().is_some_and(|(level, _)| *level >= indent) {
            stack.pop();
        }
        let value = value.split(" #").next().unwrap_or("").trim();
        let value = value.trim_matches(|c| c == '"' || c == '\'');
        let key = key.trim().to_string();
        if !value.is_empty() {
            let path: Vec<&str> = stack
                .iter()
                .map(|(_, name)| name.as_str())
                .chain([key.as_str()])
                .collect();
            map.insert(path.join("."), value.to_string());
        }
        stack.push((indent, key));
    }
    map
}

/// Formato do apelido: `^[a-z][a-z0-9._-]*(\[[a-z0-9=,._-]+\])?$`.
pub fn is_alias(value: &str) -> bool {
    let (name, suffix) = match value.find('[') {
        Some(index) => (&value[..index], Some(&value[index..])),
        None => (value, None),
    };
    let mut chars = name.chars();
    let head = chars.next().is_some_and(|c| c.is_ascii_lowercase());
    let body = chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "._-".contains(c));
    let tail = match suffix {
        None => true,
        Some(text) => text
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
            .is_some_and(|inner| {
                !inner.is_empty()
                    && inner.chars().all(|c| {
                        c.is_ascii_lowercase() || c.is_ascii_digit() || "=,._-".contains(c)
                    })
            }),
    };
    head && body && tail
}

fn override_errors(project: &BTreeMap<String, String>) -> Vec<String> {
    let mut errors = Vec::new();
    for (key, value) in project {
        let Some(rest) = key.strip_prefix("overrides.") else {
            continue;
        };
        let (profile, _tool) = rest.split_once('.').unwrap_or((rest, ""));
        if !PROFILES.contains(&profile) {
            errors.push(format!(
                "perfil desconhecido em {MODELS_FILE} (overrides.{profile}); use mecanica, padrao ou julgamento"
            ));
        } else if !is_alias(value) {
            errors.push(format!(
                "apelido inválido em {MODELS_FILE} ({key}): '{value}'; use só letras minúsculas, números, '.', '-' e '_', começando por letra, com sufixo opcional entre colchetes (ex.: sonnet)"
            ));
        }
    }
    errors
}

pub fn resolve(
    shipped: &str,
    project: &str,
    tool: Option<&str>,
    profile: &str,
) -> Result<ProfileModel> {
    let project = flatten(project);
    let errors = override_errors(&project);
    if !errors.is_empty() {
        bail!("{}", errors.join("\n"));
    }
    if !PROFILES.contains(&profile) {
        return Ok(ProfileModel {
            profile: profile.to_string(),
            description: "-".to_string(),
            tool: tool.map(str::to_string),
            model: SESSION_MODEL.to_string(),
        });
    }
    let shipped = flatten(shipped);
    let model = tool.and_then(|tool| {
        project
            .get(&format!("overrides.{profile}.{tool}"))
            .or_else(|| shipped.get(&format!("profiles.{profile}.fallback.{tool}")))
            .cloned()
    });
    Ok(ProfileModel {
        profile: profile.to_string(),
        description: "-".to_string(),
        tool: tool.map(str::to_string),
        model: model.unwrap_or_else(|| SESSION_MODEL.to_string()),
    })
}

/// Ferramenta de `harness.tool` em `config/main.yml`; `None` se ausente, vazia ou desconhecida.
pub fn harness_tool(harness: &Harness) -> Option<String> {
    let value = config_get(&harness.config_file("main.yml"), "harness.tool").ok()?;
    TOOLS.contains(&value.as_str()).then_some(value)
}

pub fn tool_warning() -> String {
    "Aviso: harness.tool vazio ou desconhecido em config/main.yml; use claude-code, codex, cursor, gemini-cli ou copilot para receber o apelido de reserva.".to_string()
}

/// Perfil resolvido para a ferramenta do projeto: descrição de `profile_descriptions` e apelido.
pub fn for_profile(harness: &Harness, profile: &str) -> Result<ProfileModel> {
    let shipped = crate::embedded::file(MODELS_FILE)
        .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
        .unwrap_or_default();
    let project = fs::read_to_string(harness.config_file("models.yml")).unwrap_or_default();
    let tool = harness_tool(harness);
    let mut info = resolve(&shipped, &project, tool.as_deref(), profile)?;
    if let Ok(description) = config_get(
        &harness.config_file("tasks.yml"),
        &format!("profile_descriptions.{profile}"),
    ) {
        if !description.is_empty() {
            info.description = description;
        }
    }
    Ok(info)
}

/// Perfil da tarefa (`profile` ou o equivalente de `effort`); `-` quando não há nenhum.
pub fn task_profile(harness: &Harness, file: &Path) -> String {
    crate::commands::plan::task_profile(harness, file).unwrap_or_else(|| "-".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHIPPED: &str = "profiles:\n  mecanica:\n    fallback:\n      claude-code: haiku\n  padrao:\n    fallback:\n      claude-code: sonnet\n  julgamento:\n    fallback:\n      claude-code: opus\n";

    fn model(project: &str, tool: Option<&str>, profile: &str) -> String {
        resolve(SHIPPED, project, tool, profile).unwrap().model
    }

    #[test]
    fn claude_code_gets_the_shipped_alias_of_each_profile() {
        let code = Some("claude-code");
        assert_eq!(model("", code, "mecanica"), "haiku");
        assert_eq!(model("", code, "padrao"), "sonnet");
        assert_eq!(model("", code, "julgamento"), "opus");
    }

    #[test]
    fn project_override_wins_over_the_shipped_alias() {
        let project = "overrides:\n  julgamento:\n    claude-code: sonnet\n";
        assert_eq!(model(project, Some("claude-code"), "julgamento"), "sonnet");
    }

    #[test]
    fn invalid_override_is_refused_with_the_key() {
        let project = "overrides:\n  julgamento:\n    claude-code: \"Opus 4!\"\n";
        let error = resolve(SHIPPED, project, Some("claude-code"), "julgamento")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("overrides.julgamento.claude-code"),
            "{error}"
        );
        assert!(error.contains("'Opus 4!'"), "{error}");
    }

    #[test]
    fn unknown_profile_in_overrides_is_refused() {
        let project = "overrides:\n  rapido:\n    claude-code: haiku\n";
        let error = resolve(SHIPPED, project, Some("claude-code"), "padrao")
            .unwrap_err()
            .to_string();
        assert!(error.contains("perfil desconhecido"), "{error}");
    }

    #[test]
    fn tool_without_alias_uses_the_session_model() {
        assert_eq!(model("", Some("gemini-cli"), "padrao"), SESSION_MODEL);
    }

    #[test]
    fn missing_tool_uses_the_session_model() {
        assert_eq!(model("", None, "padrao"), SESSION_MODEL);
    }

    #[test]
    fn unknown_profile_uses_the_session_model() {
        let info = resolve(SHIPPED, "", Some("claude-code"), "-").unwrap();
        assert_eq!(info.description, "-");
        assert_eq!(info.model, SESSION_MODEL);
    }

    #[test]
    fn alias_format() {
        for ok in [
            "haiku",
            "sonnet[1m]",
            "claude-opus-5[effort=high]",
            "composer-2.5[fast=false]",
            "gpt-5.1-codex",
            "o3_mini",
            "claude-sonnet-4-5",
        ] {
            assert!(is_alias(ok), "{ok}");
        }
        for bad in [
            "", "Opus", "opus 4", "-opus", "a|b", "opus:4", "opus[]", "opus[1m",
        ] {
            assert!(!is_alias(bad), "{bad}");
        }
    }
}
