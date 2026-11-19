//! Coletor e comparador do benchmark do fluxo do Vostok.
//! Uso: `fluxo coletar <pasta da repetição> | fluxo comparar <cenário>`.

use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Erro com código de saída próprio.
#[derive(Debug)]
struct Saida {
    codigo: u8,
    mensagem: String,
}

impl fmt::Display for Saida {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.mensagem)
    }
}

impl std::error::Error for Saida {}

fn sair(codigo: u8, mensagem: impl Into<String>) -> anyhow::Error {
    Saida {
        codigo,
        mensagem: mensagem.into(),
    }
    .into()
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let resultado = match args.as_slice() {
        [c, pasta] if c == "coletar" => coletar(Path::new(pasta)),
        [c, cenario] if c == "comparar" => comparar(cenario),
        _ => Err(sair(
            2,
            "uso: fluxo coletar <pasta da repetição> | fluxo comparar <cenário>",
        )),
    };
    match resultado {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            if let Some(s) = e.downcast_ref::<Saida>() {
                eprintln!("{}", s.mensagem);
                ExitCode::from(s.codigo)
            } else {
                eprintln!("{e:#}");
                ExitCode::from(1)
            }
        }
    }
}

// ------------------------------------------------------------------ coletor

struct Execucao {
    braco: String,
    modelo: String,
    falhou: bool,
    motivo: String,
}

/// Lê `execucao.txt` (`chave=valor`); ausente, a repetição conta como falha.
fn ler_execucao(pasta: &Path) -> Execucao {
    let mut e = Execucao {
        braco: String::new(),
        modelo: String::new(),
        falhou: true,
        motivo: "sem execucao.txt".into(),
    };
    let Ok(texto) = fs::read_to_string(pasta.join("execucao.txt")) else {
        return e;
    };
    e.falhou = false;
    e.motivo = String::new();
    for linha in texto.lines() {
        if let Some((k, v)) = linha.split_once('=') {
            match k {
                "braco" => e.braco = v.to_string(),
                "modelo" => e.modelo = v.to_string(),
                "falhou" => e.falhou = v.trim() == "true",
                "motivo" => e.motivo = v.to_string(),
                _ => {}
            }
        }
    }
    e
}

#[derive(Default)]
struct Tokens {
    input: u64,
    output: u64,
    cache_read: u64,
    cache_creation: u64,
}

impl Tokens {
    fn total(&self) -> u64 {
        self.input + self.output + self.cache_read + self.cache_creation
    }
}

fn arquivos_jsonl(dir: &Path, saida: &mut Vec<PathBuf>) {
    let Ok(entradas) = fs::read_dir(dir) else {
        return;
    };
    let mut caminhos: Vec<PathBuf> = entradas.flatten().map(|e| e.path()).collect();
    caminhos.sort();
    for c in caminhos {
        if c.is_dir() {
            arquivos_jsonl(&c, saida);
        } else if c.extension().is_some_and(|x| x == "jsonl") {
            saida.push(c);
        }
    }
}

/// Converte `AAAA-MM-DDTHH:MM:SS[.fração]Z` em segundos desde 1970 (dias civis).
fn segundos_iso(ts: &str) -> Option<i64> {
    let b = ts.as_bytes();
    if b.len() < 19
        || b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
    {
        return None;
    }
    let n = |a: usize, z: usize| ts.get(a..z)?.parse::<i64>().ok();
    let (ano, mes, dia) = (n(0, 4)?, n(5, 7)?, n(8, 10)?);
    let (h, mi, s) = (n(11, 13)?, n(14, 16)?, n(17, 19)?);
    let a = if mes <= 2 { ano - 1 } else { ano };
    let era = a.div_euclid(400);
    let yoe = a - era * 400;
    let mp = (mes + 9) % 12;
    let doy = (153 * mp + 2) / 5 + dia - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let dias = era * 146097 + doe - 719468;
    Some(dias * 86400 + h * 3600 + mi * 60 + s)
}

/// Soma os tokens uma vez por resposta; devolve também as linhas `assistant`
/// sem uso e o tempo do primeiro ao último registro.
fn somar_tokens(dir: &Path) -> (Tokens, u64, i64) {
    let mut arquivos = Vec::new();
    arquivos_jsonl(dir, &mut arquivos);
    let mut vistos: HashSet<String> = HashSet::new();
    let mut t = Tokens::default();
    let mut sem_uso = 0u64;
    let (mut menor, mut maior): (Option<i64>, Option<i64>) = (None, None);
    for arq in arquivos {
        let Ok(texto) = fs::read_to_string(&arq) else {
            continue;
        };
        for linha in texto.lines().filter(|l| !l.trim().is_empty()) {
            let Ok(v) = serde_json::from_str::<Value>(linha) else {
                continue;
            };
            if let Some(s) = v
                .get("timestamp")
                .and_then(Value::as_str)
                .and_then(segundos_iso)
            {
                menor = Some(menor.map_or(s, |m| m.min(s)));
                maior = Some(maior.map_or(s, |m| m.max(s)));
            }
            let msg = v.get("message");
            if let Some(uso) = msg.and_then(|m| m.get("usage")).filter(|u| u.is_object()) {
                let id = msg
                    .and_then(|m| m.get("id"))
                    .or_else(|| v.get("requestId"))
                    .or_else(|| v.get("uuid"))
                    .and_then(Value::as_str);
                if let Some(id) = id {
                    if !vistos.insert(id.to_string()) {
                        continue;
                    }
                }
                let campo = |k: &str| uso.get(k).and_then(Value::as_u64).unwrap_or(0);
                t.input += campo("input_tokens");
                t.output += campo("output_tokens");
                t.cache_read += campo("cache_read_input_tokens");
                t.cache_creation += campo("cache_creation_input_tokens");
            } else if v.get("type").and_then(Value::as_str) == Some("assistant") {
                sem_uso += 1;
            }
        }
    }
    let tempo = match (menor, maior) {
        (Some(a), Some(b)) => b - a,
        _ => 0,
    };
    (t, sem_uso, tempo)
}

struct Defeito {
    id: String,
    tarefa: String,
    arquivo: String,
    trecho: Option<String>,
    correcao: Option<String>,
}

/// Lê `gabarito.yml`: `spec:` no topo e `defeitos:` com itens `- id:`.
fn ler_gabarito(cenario: &Path) -> Result<(String, Vec<Defeito>)> {
    let caminho = cenario.join("gabarito.yml");
    let texto = fs::read_to_string(&caminho)
        .with_context(|| format!("não foi possível ler {}", caminho.display()))?;
    let mut spec = String::new();
    let mut defeitos: Vec<Defeito> = Vec::new();
    for linha in texto.lines() {
        let aparada = linha.trim_start();
        if aparada.is_empty() || aparada.starts_with('#') {
            continue;
        }
        if !linha.starts_with(' ') {
            if let Some(v) = linha.strip_prefix("spec: ") {
                spec = v.trim().to_string();
            }
            continue;
        }
        if let Some(id) = aparada.strip_prefix("- id: ") {
            defeitos.push(Defeito {
                id: id.trim_end().to_string(),
                tarefa: String::new(),
                arquivo: String::new(),
                trecho: None,
                correcao: None,
            });
        } else if let (Some(d), Some((k, v))) = (defeitos.last_mut(), aparada.split_once(": ")) {
            let v = v.trim_end().to_string();
            match k {
                "tarefa" => d.tarefa = v,
                "arquivo" => d.arquivo = v,
                "trecho" => d.trecho = Some(v),
                "correcao" => d.correcao = Some(v),
                _ => {}
            }
        }
    }
    Ok((spec, defeitos))
}

/// Com `trecho`, o texto sumiu do arquivo; com `correcao`, ele apareceu.
fn corrigido(fotos: &Path, spec: &str, d: &Defeito) -> bool {
    let caminho = fotos.join(spec).join(&d.tarefa).join(&d.arquivo);
    let Ok(bytes) = fs::read(caminho) else {
        return false;
    };
    let texto = String::from_utf8_lossy(&bytes);
    match (&d.trecho, &d.correcao) {
        (Some(t), _) => !texto.contains(t.as_str()),
        (None, Some(c)) => texto.contains(c.as_str()),
        _ => false,
    }
}

/// Ajuste, retomada, bloqueio ou resultado `failed`/`blocked` depois do `início`.
fn eventos_na_execucao(fim: &Path, spec: &str, tarefa: &str) -> bool {
    let Ok(texto) = fs::read_to_string(fim.join(spec).join(tarefa).join("registro.md")) else {
        return false;
    };
    let mut iniciou = false;
    for linha in texto.lines().filter(|l| l.starts_with('|')) {
        let cols: Vec<&str> = linha.split('|').map(str::trim).collect();
        let (Some(etapa), Some(resultado)) = (cols.get(3), cols.get(4)) else {
            continue;
        };
        if *etapa == "início" {
            iniciou = true;
        } else if iniciou
            && (matches!(*etapa, "ajuste" | "retomada" | "bloqueio")
                || matches!(*resultado, "failed" | "blocked"))
        {
            return true;
        }
    }
    false
}

#[derive(Clone, Copy, PartialEq)]
enum Situacao {
    AchadoAntes,
    Escape,
    NaoAchado,
}

impl Situacao {
    fn nome(self) -> &'static str {
        match self {
            Situacao::AchadoAntes => "achado_antes",
            Situacao::Escape => "escape",
            Situacao::NaoAchado => "nao_achado",
        }
    }
}

fn classificar(
    pasta: &Path,
    spec: &str,
    defeitos: &[Defeito],
    falhou: bool,
) -> Result<Vec<Option<Situacao>>> {
    let (inicio, antes, fim) = (
        pasta.join("tarefas-inicio"),
        pasta.join("tarefas-antes"),
        pasta.join("tarefas-fim"),
    );
    if falhou || !antes.is_dir() || !fim.is_dir() {
        return Ok(vec![None; defeitos.len()]);
    }
    defeitos
        .iter()
        .map(|d| {
            if corrigido(&inicio, spec, d) {
                return Err(sair(
                    1,
                    format!("trecho do gabarito ausente na referência: {}", d.id),
                ));
            }
            Ok(Some(if corrigido(&antes, spec, d) {
                Situacao::AchadoAntes
            } else if corrigido(&fim, spec, d) || eventos_na_execucao(&fim, spec, &d.tarefa) {
                Situacao::Escape
            } else {
                Situacao::NaoAchado
            }))
        })
        .collect()
}

fn coletar(pasta: &Path) -> Result<()> {
    let pasta = pasta
        .canonicalize()
        .with_context(|| format!("pasta da repetição inexistente: {}", pasta.display()))?;
    let cenario = pasta
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .context("pasta da repetição fora de <cenário>/resultados/<braço>/<n>")?;
    let exec = ler_execucao(&pasta);
    let (spec, defeitos) = ler_gabarito(cenario)?;
    let (tokens, sem_uso, tempo) = somar_tokens(&pasta.join("transcricoes"));
    let situacoes = classificar(&pasta, &spec, &defeitos, exec.falhou)?;
    let conta = |s: Situacao| situacoes.iter().filter(|x| **x == Some(s)).count();

    let metricas = json!({
        "braco": exec.braco,
        "modelo": exec.modelo,
        "tokens": {
            "input": tokens.input,
            "output": tokens.output,
            "cache_read": tokens.cache_read,
            "cache_creation": tokens.cache_creation,
            "total": tokens.total(),
        },
        "tempo_s": tempo,
        "defeitos": defeitos.len(),
        "achados_antes": conta(Situacao::AchadoAntes),
        "escapes": conta(Situacao::Escape),
        "nao_achados": conta(Situacao::NaoAchado),
        "linhas_sem_uso": sem_uso,
        "falhou": exec.falhou,
        "motivo": exec.motivo,
    });
    let texto = serde_json::to_string_pretty(&metricas)?;
    fs::write(pasta.join("metricas.json"), format!("{texto}\n"))?;

    let mut md = String::from("| Defeito | Tarefa | Situação |\n|---|---|---|\n");
    for (d, s) in defeitos.iter().zip(&situacoes) {
        md.push_str(&format!(
            "| {} | {} | {} |\n",
            d.id,
            d.tarefa,
            s.map_or("sem classificação", Situacao::nome)
        ));
    }
    fs::write(pasta.join("defeitos.md"), md)?;
    println!("{texto}");
    Ok(())
}

// --------------------------------------------------------------- comparador

struct Rep {
    modelo: String,
    total: f64,
    input: f64,
    output: f64,
    cache_read: f64,
    cache_creation: f64,
    tempo: f64,
    achados: f64,
    escapes: f64,
    sem_uso: u64,
    falhou: bool,
}

fn ler_reps(dir: &Path) -> Vec<Rep> {
    let Ok(entradas) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut pastas: Vec<PathBuf> = entradas.flatten().map(|e| e.path()).collect();
    pastas.sort();
    pastas
        .iter()
        .filter_map(|p| {
            let v: Value =
                serde_json::from_str(&fs::read_to_string(p.join("metricas.json")).ok()?).ok()?;
            let num = |v: &Value| v.as_f64().unwrap_or(0.0);
            Some(Rep {
                modelo: v["modelo"].as_str().unwrap_or("").to_string(),
                total: num(&v["tokens"]["total"]),
                input: num(&v["tokens"]["input"]),
                output: num(&v["tokens"]["output"]),
                cache_read: num(&v["tokens"]["cache_read"]),
                cache_creation: num(&v["tokens"]["cache_creation"]),
                tempo: num(&v["tempo_s"]),
                achados: num(&v["achados_antes"]),
                escapes: num(&v["escapes"]),
                sem_uso: v["linhas_sem_uso"].as_u64().unwrap_or(0),
                falhou: v["falhou"].as_bool().unwrap_or(false),
            })
        })
        .collect()
}

fn mediana(mut xs: Vec<f64>) -> Option<f64> {
    if xs.is_empty() {
        return None;
    }
    xs.sort_by(|a, b| a.total_cmp(b));
    let n = xs.len();
    Some(if n % 2 == 1 {
        xs[n / 2]
    } else {
        (xs[n / 2 - 1] + xs[n / 2]) / 2.0
    })
}

/// Uma casa decimal, vírgula e sem `,0` final.
fn numero(x: Option<f64>) -> String {
    match x {
        None => "-".into(),
        Some(x) => {
            let s = format!("{x:.1}").replace('.', ",");
            s.strip_suffix(",0").map(str::to_string).unwrap_or(s)
        }
    }
}

fn variacao(sem: Option<f64>, com: Option<f64>) -> String {
    match (sem, com) {
        (Some(s), Some(c)) if s != 0.0 => format!("{:+.1}%", (c - s) / s * 100.0).replace('.', ","),
        _ => "-".into(),
    }
}

/// Nome da métrica e como lê o valor de uma repetição.
type Metrica = (&'static str, fn(&Rep) -> f64);

fn validas(r: &[Rep]) -> Vec<&Rep> {
    r.iter().filter(|x| !x.falhou).collect()
}

fn comparar(cenario: &str) -> Result<()> {
    let base = match std::env::var_os("FLUXO_BENCHMARKS").filter(|v| !v.is_empty()) {
        Some(v) => PathBuf::from(v),
        None => Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default(),
    };
    let pasta = base.join(cenario);
    let sem = ler_reps(&pasta.join("resultados/sem"));
    let com = ler_reps(&pasta.join("resultados/com"));
    if sem.is_empty() && com.is_empty() {
        return Err(sair(2, format!("sem resultados em {}", pasta.display())));
    }
    let (vs, vc) = (validas(&sem), validas(&com));
    let med = |r: &[&Rep], f: fn(&Rep) -> f64| mediana(r.iter().map(|x| f(x)).collect());

    println!(
        "Repetições válidas: sem {} de {}, com {} de {}\n",
        vs.len(),
        sem.len(),
        vc.len(),
        com.len()
    );
    println!("| Métrica | sem (mediana) | com (mediana) | Variação |\n|---|---|---|---|");
    let linhas: [Metrica; 8] = [
        ("Tokens totais", |r| r.total),
        ("Tempo total (s)", |r| r.tempo),
        ("Achados antes da execução", |r| r.achados),
        ("Escapes", |r| r.escapes),
        ("Tokens de entrada", |r| r.input),
        ("Tokens de saída", |r| r.output),
        ("Leitura de cache", |r| r.cache_read),
        ("Gravação de cache", |r| r.cache_creation),
    ];
    for (nome, f) in linhas {
        let (s, c) = (med(&vs, f), med(&vc, f));
        println!(
            "| {nome} | {} | {} | {} |",
            numero(s),
            numero(c),
            variacao(s, c)
        );
    }
    println!(
        "\nLinhas sem uso: sem {}, com {}",
        sem.iter().map(|r| r.sem_uso).sum::<u64>(),
        com.iter().map(|r| r.sem_uso).sum::<u64>()
    );

    println!();
    let modelos: HashSet<&str> = vs
        .iter()
        .chain(vc.iter())
        .map(|r| r.modelo.as_str())
        .collect();
    if vs.len() < 5 || vc.len() < 5 {
        let (braco, n) = if vs.len() < 5 {
            ("sem", vs.len())
        } else {
            ("com", vc.len())
        };
        println!("Decisão: INCONCLUSIVO");
        println!("Motivo: menos de 5 repetições válidas ({braco}: {n})");
    } else if modelos.len() > 1 {
        println!("Decisão: INCONCLUSIVO");
        println!("Motivo: modelos diferentes entre os braços");
    } else {
        let g = |r: &[&Rep], f: fn(&Rep) -> f64| med(r, f).unwrap_or(0.0);
        let melhora = g(&vc, |r| r.escapes) < g(&vs, |r| r.escapes)
            || g(&vc, |r| r.achados) > g(&vs, |r| r.achados);
        let tokens_ok = g(&vc, |r| r.total) * 100.0 <= g(&vs, |r| r.total) * 110.0;
        let tempo_ok = g(&vc, |r| r.tempo) * 100.0 <= g(&vs, |r| r.tempo) * 110.0;
        println!(
            "Decisão: {}",
            if melhora && tokens_ok && tempo_ok {
                "VERSÃO"
            } else {
                "REVERTER"
            }
        );
    }
    Ok(())
}
