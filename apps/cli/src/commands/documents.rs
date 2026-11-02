//! Comandos `documents`: identidade visual dos documentos finais.
//!
//! O `brand` grava a identidade do repositório em `config/documents.yml` e deriva dela os tokens de
//! cor dos dois temas, garantindo contraste 4,5:1 (WCAG AA) nos pares de texto. As cores de estado
//! (sucesso, alerta, perigo e informação) nunca mudam.

use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use clap::{Args, Subcommand};

use crate::embedded;
use crate::harness::Harness;

const MIN_CONTRAST: f64 = 4.5;
const ICON_EXTENSIONS: [&str; 3] = ["svg", "png", "webp"];
const ICON_MAX_BYTES: u64 = 256 * 1024;
const SECONDARY_HUE_SHIFT: f64 = -60.0;
const DARK_PRIMARY_MIN_LIGHTNESS: f64 = 65.0;
const MAX_NEUTRAL_SATURATION: f64 = 40.0;
const THEMES: [&str; 2] = ["light", "dark"];
const BRAND_KEYS: [&str; 4] = ["name", "icon", "primary", "secondary"];
const BRAND_BLOCK: &str = "\n# Identidade do repositório, gravada pelo `vostok documents brand`. Vazio usa a do harness.\nbrand:\n  name: \"\"\n  icon: \"\"        # caminho relativo à raiz do projeto: .svg, .png ou .webp, até 256 KB\n  primary: \"\"\n  secondary: \"\"\n";
const NEUTRALS: [&str; 10] = [
    "bg",
    "bg_elevated",
    "surface_muted",
    "surface_hover",
    "border",
    "border_strong",
    "text",
    "text_muted",
    "text_faint",
    "code_bg",
];
const DERIVED_ORDER: [&str; 22] = [
    "primary",
    "primary_hover",
    "primary_soft",
    "primary_on",
    "accent",
    "accent_soft",
    "brand_from",
    "brand_to",
    "calm_from",
    "calm_to",
    "bg",
    "bg_elevated",
    "surface_muted",
    "surface_hover",
    "border",
    "border_strong",
    "text",
    "text_muted",
    "text_faint",
    "code_bg",
    "shadow_sm",
    "shadow_md",
];

#[derive(Debug, Subcommand)]
pub enum DocumentsCommand {
    /// Grava a identidade do repositório e deriva as cores dos documentos.
    Brand(BrandArgs),
}

#[derive(Debug, Args)]
pub struct BrandArgs {
    /// Cor principal (#rgb ou #rrggbb).
    #[arg(long, conflicts_with = "reset")]
    primary: Option<String>,
    /// Cor secundária; sem ela, a principal com o matiz girado em -60°.
    #[arg(long, requires = "primary")]
    secondary: Option<String>,
    /// Nome do repositório nos documentos.
    #[arg(long, conflicts_with = "reset")]
    name: Option<String>,
    /// Ícone (.svg, .png ou .webp, até 256 KB), relativo à raiz do projeto.
    #[arg(long, conflicts_with = "reset")]
    icon: Option<String>,
    /// Volta à identidade padrão do harness.
    #[arg(long)]
    reset: bool,
}

pub fn run(command: DocumentsCommand) -> Result<()> {
    match command {
        DocumentsCommand::Brand(args) => brand(args),
    }
}

// --- Cores ---

type Rgb = (u8, u8, u8);

#[derive(Debug, Clone, Copy)]
struct Hsl {
    hue: f64,
    saturation: f64,
    lightness: f64,
}

impl Hsl {
    fn with_lightness(self, lightness: f64) -> Self {
        Self {
            lightness: lightness.clamp(0.0, 100.0),
            ..self
        }
    }

    fn with_hue(self, hue: f64) -> Self {
        Self {
            hue: hue.rem_euclid(360.0),
            ..self
        }
    }
}

fn parse_hex(text: &str) -> Option<Rgb> {
    let digits = text.strip_prefix('#')?;
    if !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let expanded: String = match digits.len() {
        3 => digits.chars().flat_map(|c| [c, c]).collect(),
        6 => digits.to_string(),
        _ => return None,
    };
    let channel = |at: usize| u8::from_str_radix(&expanded[at..at + 2], 16).ok();
    Some((channel(0)?, channel(2)?, channel(4)?))
}

fn to_hex((r, g, b): Rgb) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

fn to_hsl((r, g, b): Rgb) -> Hsl {
    let [r, g, b] = [r, g, b].map(|channel| f64::from(channel) / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let lightness = (max + min) / 2.0;
    let delta = max - min;
    if delta == 0.0 {
        return Hsl {
            hue: 0.0,
            saturation: 0.0,
            lightness: lightness * 100.0,
        };
    }
    let saturation = delta / (1.0 - (2.0 * lightness - 1.0).abs());
    let sector = if max == r {
        ((g - b) / delta).rem_euclid(6.0)
    } else if max == g {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };
    Hsl {
        hue: sector * 60.0,
        saturation: saturation * 100.0,
        lightness: lightness * 100.0,
    }
}

fn to_rgb(color: Hsl) -> Rgb {
    let saturation = color.saturation / 100.0;
    let lightness = color.lightness / 100.0;
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let sector = color.hue / 60.0;
    let second = chroma * (1.0 - (sector % 2.0 - 1.0).abs());
    let (r, g, b) = match sector as u32 {
        0 => (chroma, second, 0.0),
        1 => (second, chroma, 0.0),
        2 => (0.0, chroma, second),
        3 => (0.0, second, chroma),
        4 => (second, 0.0, chroma),
        _ => (chroma, 0.0, second),
    };
    let offset = lightness - chroma / 2.0;
    let channel = |value: f64| ((value + offset) * 255.0).round().clamp(0.0, 255.0) as u8;
    (channel(r), channel(g), channel(b))
}

fn luminance((r, g, b): Rgb) -> f64 {
    let linear = |channel: u8| {
        let value = f64::from(channel) / 255.0;
        if value <= 0.03928 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
}

fn contrast(a: Rgb, b: Rgb) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// Resultado de `readable`: a cor final e se a claridade precisou mudar.
struct Readable {
    color: Rgb,
    adjusted: bool,
}

/// Muda a claridade de 1 em 1 ponto (`step` negativo escurece, positivo clareia) até a cor passar
/// o contraste mínimo contra todos os fundos, ou até esgotar a escala.
fn readable(start: Hsl, backgrounds: &[Rgb], step: f64) -> Readable {
    let mut candidate = start;
    let mut adjusted = false;
    loop {
        let color = to_rgb(candidate);
        let passes = backgrounds
            .iter()
            .all(|bg| contrast(color, *bg) >= MIN_CONTRAST);
        let next = candidate.lightness + step;
        if passes || !(0.0..=100.0).contains(&next) {
            return Readable { color, adjusted };
        }
        candidate = candidate.with_lightness(next);
        adjusted = true;
    }
}

// --- Derivação ---

struct Theme<'a> {
    name: &'static str,
    light: bool,
    defaults: &'a [(String, String)],
}

struct Derived {
    tokens: Vec<(&'static str, String)>,
    warnings: Vec<String>,
}

impl Theme<'_> {
    fn default_of(&self, token: &str) -> Result<&str> {
        self.defaults
            .iter()
            .find(|(key, _)| key == token)
            .map(|(_, value)| value.as_str())
            .ok_or_else(|| {
                anyhow!(
                    "o token '{}.{token}' falta no documents.yml embutido",
                    self.name
                )
            })
    }

    fn default_color(&self, token: &str) -> Result<Rgb> {
        parse_hex(self.default_of(token)?)
            .ok_or_else(|| anyhow!("o token '{}.{token}' padrão não é uma cor", self.name))
    }

    fn step(&self) -> f64 {
        if self.light {
            -1.0
        } else {
            1.0
        }
    }

    fn derive(&self, primary: Rgb, secondary: Rgb) -> Result<Derived> {
        let (p, s) = (to_hsl(primary), to_hsl(secondary));
        let mut tokens: Vec<(&'static str, String)> = Vec::new();
        let mut warnings = Vec::new();

        let neutral = |token: &'static str| -> Result<Rgb> {
            let base = to_hsl(self.default_color(token)?);
            let tinted = Hsl {
                hue: p.hue,
                saturation: base.saturation.min(MAX_NEUTRAL_SATURATION),
                lightness: base.lightness,
            };
            Ok(to_rgb(tinted))
        };
        let bg = neutral("bg")?;
        let surface = self.default_color("surface")?;
        let backgrounds = [bg, surface];

        let mut checked = |token: &'static str, start: Hsl, informed: Rgb| {
            let result = readable(start, &backgrounds, self.step());
            if result.adjusted {
                warnings.push(format!(
                    "aviso: {}.{token} ajustado de {} para {} para contraste 4,5:1",
                    self.name,
                    to_hex(informed),
                    to_hex(result.color)
                ));
            }
            result.color
        };

        let mut texts = Vec::new();
        for token in ["text", "text_muted"] {
            let start = to_hsl(neutral(token)?);
            texts.push((token, checked(token, start, neutral(token)?)));
        }
        let text = texts[0].1;
        let mut accents = Vec::new();
        for (token, soft, color) in [("primary", "primary_soft", p), ("accent", "accent_soft", s)] {
            let start = if self.light {
                color
            } else {
                color.with_lightness(color.lightness.max(DARK_PRIMARY_MIN_LIGHTNESS))
            };
            let informed = to_rgb(color);
            accents.push((token, soft, color, checked(token, start, informed)));
        }
        let [(_, _, _, primary_color), (_, _, _, accent_color)] = [accents[0], accents[1]];

        let soft = |color: Hsl| {
            let (scale, lightness) = if self.light { (0.7, 93.0) } else { (0.6, 20.0) };
            to_hex(to_rgb(Hsl {
                saturation: color.saturation * scale,
                lightness,
                ..color
            }))
        };
        let calm = |color: Hsl| {
            let (saturation, lightness) = if self.light {
                (80.0, 96.0)
            } else {
                (50.0, 14.0)
            };
            to_hex(to_rgb(Hsl {
                saturation,
                lightness,
                ..color
            }))
        };
        let hover_step = self.step() * 8.0;
        let primary_hsl = to_hsl(primary_color);
        let primary_on = if self.light {
            if contrast((255, 255, 255), primary_color) >= MIN_CONTRAST {
                (255, 255, 255)
            } else {
                text
            }
        } else {
            bg
        };

        tokens.push(("primary", to_hex(primary_color)));
        tokens.push((
            "primary_hover",
            to_hex(to_rgb(
                primary_hsl.with_lightness(primary_hsl.lightness + hover_step),
            )),
        ));
        tokens.push(("primary_soft", soft(p)));
        tokens.push(("primary_on", to_hex(primary_on)));
        tokens.push(("accent", to_hex(accent_color)));
        tokens.push(("accent_soft", soft(s)));
        tokens.push(("brand_from", to_hex(primary)));
        tokens.push(("brand_to", to_hex(secondary)));
        tokens.push(("calm_from", calm(p)));
        tokens.push(("calm_to", calm(s)));
        for token in NEUTRALS {
            let value = match texts.iter().find(|(name, _)| *name == token) {
                Some((_, color)) => *color,
                None => neutral(token)?,
            };
            tokens.push((token, to_hex(value)));
        }
        let shadow_rgb = to_rgb(p.with_lightness(20.0));
        for token in ["shadow_sm", "shadow_md"] {
            let default = self.default_of(token)?;
            let value = if self.light {
                recolor_shadow(default, shadow_rgb)
            } else {
                default.to_string()
            };
            tokens.push((token, value));
        }
        Ok(Derived { tokens, warnings })
    }
}

/// Troca o RGB de cada `rgba(r, g, b, a)` da sombra, mantendo o alfa.
fn recolor_shadow(shadow: &str, (r, g, b): Rgb) -> String {
    let mut parts = shadow.split("rgba(");
    let mut out = parts.next().unwrap_or_default().to_string();
    for part in parts {
        let alpha = part.splitn(4, ',').nth(3).unwrap_or(part);
        out.push_str(&format!("rgba({r}, {g}, {b},{alpha}"));
    }
    out
}

// --- Arquivo de configuração ---

/// Troca de valor: grupo, chave e valor novo.
type Update = (String, String, String);

fn update(group: &str, key: &str, value: impl Into<String>) -> Update {
    (group.to_string(), key.to_string(), value.into())
}

/// Pares `chave: "valor"` de um grupo de primeiro nível.
fn group_values(content: &str, group: &str) -> Vec<(String, String)> {
    let header = format!("{group}:");
    let mut inside = false;
    let mut values = Vec::new();
    for line in content.lines() {
        if !line.is_empty() && !line.starts_with([' ', '\t', '#']) {
            inside = line.trim_end() == header;
            continue;
        }
        if !inside {
            continue;
        }
        let Some((key, rest)) = line.trim().split_once(':') else {
            continue;
        };
        if let Some(value) = rest.trim().strip_prefix('"') {
            if let Some((value, _)) = value.split_once('"') {
                values.push((key.to_string(), value.to_string()));
            }
        }
    }
    values
}

fn lookup<'a>(values: &'a [(String, String)], key: &str) -> Option<&'a str> {
    values
        .iter()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.as_str())
}

/// Reescreve só o valor das linhas `  chave: "..."` dos grupos citados, preservando o resto.
fn rewrite(content: &str, updates: &[Update]) -> String {
    let mut group = "";
    let mut lines: Vec<String> = Vec::new();
    for line in content.lines() {
        if !line.is_empty() && !line.starts_with([' ', '\t', '#']) {
            group = line.trim_end().trim_end_matches(':');
        }
        lines.push(rewrite_line(line, group, updates));
    }
    let mut out = lines.join("\n");
    if content.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn rewrite_line(line: &str, group: &str, updates: &[Update]) -> String {
    let Some((key, rest)) = line.split_once(':') else {
        return line.to_string();
    };
    let indented = line.starts_with("  ");
    let Some((_, _, value)) = updates
        .iter()
        .find(|(g, k, _)| indented && g == group && k == key.trim())
    else {
        return line.to_string();
    };
    let Some(after_quote) = rest.trim_start().strip_prefix('"') else {
        return line.to_string();
    };
    let Some((_, tail)) = after_quote.split_once('"') else {
        return line.to_string();
    };
    format!("{key}: \"{value}\"{tail}")
}

// --- brand ---

fn brand(args: BrandArgs) -> Result<()> {
    if !args.reset && args.primary.is_none() && args.name.is_none() && args.icon.is_none() {
        bail!("Uso: vostok documents brand --primary <cor> [--secondary <cor>] [--name <nome>] [--icon <caminho>] | --reset");
    }
    let harness = Harness::from_current_dir()?;
    let path = harness.config_file("documents.yml");
    let content = fs::read_to_string(&path)
        .with_context(|| format!("não foi possível ler {}", path.display()))?;
    let embedded = embedded_documents()?;

    let mut updates: Vec<Update> = Vec::new();
    let mut changes: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    if args.reset {
        for theme in THEMES {
            let current = group_values(&content, theme);
            for (token, default) in group_values(&embedded, theme) {
                if let Some(before) = lookup(&current, &token) {
                    note_change(&mut changes, theme, &token, before, &default);
                    updates.push(update(theme, &token, default));
                }
            }
        }
        for key in BRAND_KEYS {
            updates.push(update("brand", key, ""));
        }
    } else {
        if let Some(icon) = &args.icon {
            validate_icon(&harness, icon)?;
            updates.push(update("brand", "icon", icon.as_str()));
        }
        if let Some(name) = &args.name {
            updates.push(update("brand", "name", name.trim()));
        }
        if let Some(primary) = &args.primary {
            let primary_rgb = parse_color("--primary", primary)?;
            let secondary_rgb = match &args.secondary {
                Some(secondary) => parse_color("--secondary", secondary)?,
                None => {
                    let hsl = to_hsl(primary_rgb);
                    to_rgb(hsl.with_hue(hsl.hue + SECONDARY_HUE_SHIFT))
                }
            };
            updates.push(update("brand", "primary", to_hex(primary_rgb)));
            let recorded_secondary = args.secondary.as_ref().map(|_| to_hex(secondary_rgb));
            updates.push(update(
                "brand",
                "secondary",
                recorded_secondary.unwrap_or_default(),
            ));
            for name in THEMES {
                let defaults = group_values(&embedded, name);
                let current = group_values(&content, name);
                let theme = Theme {
                    name,
                    light: name == "light",
                    defaults: &defaults,
                };
                let derived = theme.derive(primary_rgb, secondary_rgb)?;
                for token in DERIVED_ORDER {
                    let before = lookup(&current, token).ok_or_else(|| {
                        anyhow!("o token '{name}.{token}' falta no documents.yml; execute `vostok update`")
                    })?;
                    let after = derived
                        .tokens
                        .iter()
                        .find(|(derived_token, _)| *derived_token == token)
                        .map(|(_, value)| value.clone())
                        .unwrap_or_default();
                    note_change(&mut changes, name, token, before, &after);
                    updates.push(update(name, token, after));
                }
                warnings.extend(derived.warnings);
            }
        }
    }

    let base = if content.lines().any(|line| line.trim_end() == "brand:") {
        content
    } else {
        format!("{}\n{BRAND_BLOCK}", content.trim_end_matches('\n'))
    };
    fs::write(&path, rewrite(&base, &updates))
        .with_context(|| format!("não foi possível gravar {}", path.display()))?;

    for line in warnings.iter().chain(&changes) {
        println!("{line}");
    }
    println!("Identidade gravada em vostok/config/documents.yml");
    Ok(())
}

fn note_change(changes: &mut Vec<String>, theme: &str, token: &str, before: &str, after: &str) {
    if before != after {
        changes.push(format!("{theme}.{token}: {before} -> {after}"));
    }
}

fn embedded_documents() -> Result<String> {
    let bytes = embedded::file("config/documents.yml")
        .ok_or_else(|| anyhow!("config/documents.yml não está embutido no binário"))?;
    Ok(String::from_utf8_lossy(bytes).into_owned())
}

fn parse_color(flag: &str, text: &str) -> Result<Rgb> {
    parse_hex(text.trim())
        .ok_or_else(|| anyhow!("{flag}: '{text}' não é uma cor; use #rgb ou #rrggbb"))
}

fn validate_icon(harness: &Harness, icon: &str) -> Result<()> {
    let extension = Path::new(icon)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    if !ICON_EXTENSIONS.contains(&extension.as_str()) {
        bail!("--icon: '{icon}' precisa ser .svg, .png ou .webp");
    }
    let project_root = harness.root.parent().unwrap_or(&harness.root);
    let metadata = fs::metadata(project_root.join(icon))
        .ok()
        .filter(|metadata| metadata.is_file())
        .ok_or_else(|| anyhow!("--icon: '{icon}' não existe a partir da raiz do projeto"))?;
    if metadata.len() > ICON_MAX_BYTES {
        bail!("--icon: '{icon}' passa de 256 KB");
    }
    Ok(())
}
