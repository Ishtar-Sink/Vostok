//! Testes do grupo `documents`: identidade visual dos documentos finais (`brand`).

mod common;

use std::collections::HashMap;
use std::fs;

use common::{failure_stderr, Project};

const MIN_CONTRAST: f64 = 4.5;
const STATE_TOKENS: [&str; 8] = [
    "success",
    "success_soft",
    "warning",
    "warning_soft",
    "danger",
    "danger_soft",
    "info",
    "info_soft",
];

fn config_path(project: &Project) -> std::path::PathBuf {
    project.harness().join("config/documents.yml")
}

fn group(content: &str, name: &str) -> HashMap<String, String> {
    let header = format!("{name}:");
    let mut values = HashMap::new();
    let mut inside = false;
    for line in content.lines() {
        if !line.starts_with([' ', '#']) && !line.is_empty() {
            inside = line == header;
            continue;
        }
        if !inside {
            continue;
        }
        if let Some((key, rest)) = line.trim().split_once(':') {
            if let Some(value) = rest.trim().strip_prefix('"') {
                values.insert(
                    key.to_string(),
                    value.split('"').next().unwrap().to_string(),
                );
            }
        }
    }
    values
}

fn read_group(project: &Project, name: &str) -> HashMap<String, String> {
    group(&fs::read_to_string(config_path(project)).unwrap(), name)
}

fn embedded_group(name: &str) -> HashMap<String, String> {
    let content = String::from_utf8(
        vostok::embedded::file("config/documents.yml")
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    group(&content, name)
}

fn rgb(hex: &str) -> [f64; 3] {
    let value = u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap();
    [(value >> 16) & 255, (value >> 8) & 255, value & 255].map(|channel| channel as f64)
}

fn luminance(hex: &str) -> f64 {
    let [r, g, b] = rgb(hex).map(|channel| {
        let c = channel / 255.0;
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    });
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn contrast(a: &str, b: &str) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

fn run_brand(project: &Project, args: &[&str]) -> String {
    let output = project
        .vostok()
        .arg("documents")
        .arg("brand")
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).unwrap()
}

#[test]
fn brand_primary_rewrites_the_brand_tokens_of_both_themes() {
    let project = Project::new();
    run_brand(&project, &["--primary", "#0f766e"]);
    let brand = read_group(&project, "brand");
    assert_eq!(brand["primary"], "#0f766e");
    for theme in ["light", "dark"] {
        let before = embedded_group(theme);
        let after = read_group(&project, theme);
        for token in ["primary", "accent", "brand_from", "calm_from", "bg"] {
            assert_ne!(before[token], after[token], "{theme}.{token}");
        }
        for token in STATE_TOKENS {
            assert_eq!(before[token], after[token], "{theme}.{token}");
        }
    }
}

#[test]
fn brand_secondary_is_derived_or_taken() {
    let project = Project::new();
    run_brand(&project, &["--primary", "#ff0000"]);
    assert_eq!(read_group(&project, "light")["brand_to"], "#ff00ff");
    run_brand(
        &project,
        &["--primary", "#ff0000", "--secondary", "#f59e0b"],
    );
    assert_eq!(read_group(&project, "light")["brand_to"], "#f59e0b");
    assert_eq!(read_group(&project, "dark")["brand_to"], "#f59e0b");
    run_brand(
        &project,
        &["--primary", "#0f766e", "--secondary", "#0F766E"],
    );
    assert_eq!(read_group(&project, "light")["brand_to"], "#0f766e");
}

#[test]
fn brand_text_pairs_pass_aa_in_both_themes() {
    for primary in ["#facc15", "#111111", "#ff00ff", "#0f766e"] {
        let project = Project::new();
        let output = run_brand(&project, &["--primary", primary]);
        if primary == "#facc15" {
            assert!(output.contains("aviso: light.primary ajustado de #facc15 para"));
        }
        for theme in ["light", "dark"] {
            let tokens = read_group(&project, theme);
            let ink = |foreground: &str, background: &str| {
                let ratio = contrast(&tokens[foreground], &tokens[background]);
                assert!(
                    ratio >= MIN_CONTRAST,
                    "{primary} {theme}: {foreground} sobre {background} = {ratio:.2}"
                );
            };
            for background in ["bg", "surface"] {
                for foreground in ["text", "text_muted", "primary", "accent"] {
                    ink(foreground, background);
                }
            }
            ink("primary_on", "primary");
        }
    }
}

#[test]
fn brand_reset_restores_the_default_tokens() {
    let project = Project::new();
    run_brand(&project, &["--primary", "#0f766e", "--name", "Atria"]);
    run_brand(&project, &["--reset"]);
    for theme in ["light", "dark"] {
        assert_eq!(read_group(&project, theme), embedded_group(theme));
    }
    let brand = read_group(&project, "brand");
    assert!(["name", "icon", "primary", "secondary"]
        .iter()
        .all(|key| brand[*key].is_empty()));
}

#[test]
fn brand_prints_each_token_before_and_after() {
    let project = Project::new();
    let before = read_group(&project, "light");
    let output = run_brand(&project, &["--primary", "#0f766e"]);
    let after = read_group(&project, "light");
    let line = format!(
        "light.primary: {} -> {}",
        before["primary"], after["primary"]
    );
    assert!(output.contains(&line), "{output}");
    assert!(output.contains("dark.bg: #0b0816 -> "), "{output}");
    assert!(!output.contains("light.success:"));
}

#[test]
fn brand_refuses_invalid_input() {
    let project = Project::new();
    fs::write(project.root().join("big.svg"), vec![b'x'; 300 * 1024]).unwrap();
    fs::write(project.root().join("x.gif"), b"GIF89a").unwrap();
    let original = fs::read(config_path(&project)).unwrap();
    let cases: [&[&str]; 5] = [
        &["--primary", "azul"],
        &["--primary", "#12"],
        &["--icon", "missing.svg"],
        &["--icon", "big.svg"],
        &["--icon", "x.gif"],
    ];
    for args in cases {
        let stderr = failure_stderr(project.vostok().arg("documents").arg("brand").args(args));
        assert!(!stderr.trim().is_empty(), "{args:?}");
        assert_eq!(
            fs::read(config_path(&project)).unwrap(),
            original,
            "{args:?}"
        );
    }

    let stale = String::from_utf8(original)
        .unwrap()
        .replacen("  bg: \"#f3f0fa\"\n", "", 1);
    fs::write(config_path(&project), &stale).unwrap();
    let stderr =
        failure_stderr(
            project
                .vostok()
                .args(["documents", "brand", "--primary", "#0f766e"]),
        );
    assert!(stderr.contains("vostok update"), "{stderr}");
    assert_eq!(fs::read_to_string(config_path(&project)).unwrap(), stale);
}

#[test]
fn brand_name_and_icon_are_recorded() {
    let project = Project::new();
    fs::write(project.root().join("logo.svg"), "<svg/>").unwrap();
    run_brand(&project, &["--name", "Atria", "--icon", "logo.svg"]);
    let brand = read_group(&project, "brand");
    assert_eq!(brand["name"], "Atria");
    assert_eq!(brand["icon"], "logo.svg");
    assert_eq!(read_group(&project, "light"), embedded_group("light"));
}

fn run_detect(project: &Project) -> String {
    let output = project
        .vostok()
        .arg("documents")
        .arg("detect")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).unwrap()
}

fn write_file(project: &Project, relative: &str, content: &str) {
    let path = project.root().join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

#[test]
fn detect_lists_css_and_tailwind_colors_by_relevance() {
    let project = Project::new();
    write_file(
        &project,
        "src/theme.css",
        ":root {\n  --color-gray: #888888;\n  --color-primary: #0F766E;\n}\n",
    );
    write_file(
        &project,
        "tailwind.config.js",
        "module.exports = {\n  theme: {\n    colors: {\n      brand: '#f59e0b',\n    },\n  },\n};\n",
    );
    let output = run_detect(&project);
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(
        lines,
        [
            "src/theme.css:3  --color-primary  #0f766e",
            "tailwind.config.js:4  brand  #f59e0b",
            "src/theme.css:2  --color-gray  #888888",
        ]
    );
}

#[test]
fn detect_ignores_generated_folders_and_secrets() {
    let project = Project::new();
    for path in [
        "node_modules/x.css",
        "target/x.css",
        "dist/x.css",
        "build/x.css",
        "vostok/x.css",
        ".cache/x.css",
    ] {
        write_file(&project, path, "--color-primary: #123456;\n");
    }
    write_file(
        &project,
        ".env",
        "PRIMARY=#123456\n--color-primary: #123456;\n",
    );
    write_file(&project, ".env.local", "--color-primary: #123456;\n");
    write_file(&project, "src/app.scss", "$brand: #abcdef;\n");
    let output = run_detect(&project);
    assert_eq!(output.trim_end(), "src/app.scss:1  $brand  #abcdef");
}

#[test]
fn detect_lists_at_most_ten_candidates() {
    let project = Project::new();
    let css: String = (0..14)
        .map(|index| format!("--color-{index:02}: #1010{index:02};\n"))
        .collect();
    write_file(&project, "src/colors.less", &css.replace("--", "@"));
    let output = run_detect(&project);
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 11);
    assert_eq!(lines[0], "src/colors.less:1  @color-00  #101000");
    assert_eq!(lines[10], "... e mais 4 cores");
}

#[test]
fn detect_without_candidates_says_so() {
    let project = Project::new();
    assert_eq!(
        run_detect(&project).trim_end(),
        "Nenhuma cor encontrada no repositório."
    );
}
