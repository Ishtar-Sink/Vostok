//! Testes do coletor, do comparador e do executor do benchmark do fluxo.
//! Os programas rodam como processo, com `claude` e `vostok` falsos.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

fn escrever(caminho: &Path, conteudo: &str) {
    fs::create_dir_all(caminho.parent().unwrap()).unwrap();
    fs::write(caminho, conteudo).unwrap();
}

fn saida(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

fn erro(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

// ---------------------------------------------------------------- coletor

fn linha_uso(id: &str, ts: &str, u: (u64, u64, u64, u64)) -> String {
    format!(
        "{{\"type\":\"assistant\",\"timestamp\":\"{ts}\",\"message\":{{\"id\":\"{id}\",\"usage\":{{\"input_tokens\":{},\"output_tokens\":{},\"cache_read_input_tokens\":{},\"cache_creation_input_tokens\":{}}}}}}}\n",
        u.0, u.1, u.2, u.3
    )
}

/// Monta `<cenario>/resultados/com/1` e devolve (cenário, repetição).
fn repeticao(tmp: &Path, execucao: &str) -> (PathBuf, PathBuf) {
    let cen = tmp.join("cen");
    let rep = cen.join("resultados/com/1");
    escrever(&rep.join("execucao.txt"), execucao);
    (cen, rep)
}

const EXEC_OK: &str = "braco=com\nmodelo=sonnet\nfalhou=false\nmotivo=\n";

fn gabarito(cen: &Path, defeitos: &[(&str, &str, &str, &str, &str)]) {
    let mut s = String::from("# gabarito\nspec: SPEC-077\ndefeitos:\n");
    for (id, tarefa, arquivo, tipo, valor) in defeitos {
        s.push_str(&format!(
            "  - id: {id}\n    tarefa: {tarefa}\n    local: Passos\n    categoria: lacuna\n    arquivo: {arquivo}\n    {tipo}: {valor}\n"
        ));
    }
    escrever(&cen.join("gabarito.yml"), &s);
}

fn foto(rep: &Path, pasta: &str, tarefa: &str, arquivo: &str, conteudo: &str) {
    escrever(
        &rep.join(pasta).join("SPEC-077").join(tarefa).join(arquivo),
        conteudo,
    );
}

fn registro(rep: &Path, tarefa: &str, linhas: &[&str]) {
    let mut s = String::from("| # | Data | Etapa | Resultado | Detalhe |\n|---|---|---|---|---|\n");
    for (i, l) in linhas.iter().enumerate() {
        s.push_str(&format!("| {} | 2026-11-19 | {} | x |\n", i + 1, l));
    }
    foto(rep, "tarefas-fim", tarefa, "registro.md", &s);
}

fn coletar(rep: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_fluxo"))
        .arg("coletar")
        .arg(rep)
        .output()
        .unwrap()
}

fn json_de(o: &Output) -> Value {
    serde_json::from_str(&saida(o)).unwrap_or_else(|e| panic!("json inválido: {e}\n{}", saida(o)))
}

#[test]
fn coletar_soma_tokens_por_resposta_e_tempo() {
    let t = TempDir::new().unwrap();
    let (cen, rep) = repeticao(t.path(), EXEC_OK);
    gabarito(&cen, &[]);
    let tr = rep.join("transcricoes");
    let mut s = String::new();
    s.push_str("{\"type\":\"user\",\"timestamp\":\"2026-11-19T10:00:00.000Z\"}\n");
    s.push_str(&linha_uso(
        "m1",
        "2026-11-19T10:00:30.000Z",
        (10, 20, 300, 40),
    ));
    s.push_str(&linha_uso(
        "m1",
        "2026-11-19T10:00:31.000Z",
        (10, 20, 300, 40),
    ));
    s.push_str("{\"type\":\"assistant\",\"timestamp\":\"2026-11-19T10:01:00.000Z\",\"message\":{\"id\":\"m2\"}}\n");
    escrever(&tr.join("s1.jsonl"), &s);
    escrever(
        &tr.join("s1/subagents/agent-a.jsonl"),
        &linha_uso("m3", "2026-11-19T10:01:10.000Z", (1, 2, 30, 4)),
    );
    escrever(
        &tr.join("s1/subagents/agent-b.jsonl"),
        &linha_uso("m4", "2026-11-19T10:02:00.250Z", (100, 200, 3000, 400)),
    );
    let o = coletar(&rep);
    assert!(o.status.success(), "{}", erro(&o));
    let v = json_de(&o);
    assert_eq!(
        v["tokens"],
        serde_json::json!({"input": 111, "output": 222, "cache_read": 3330, "cache_creation": 444, "total": 4107})
    );
    assert_eq!(v["tempo_s"], 120);
    assert_eq!(v["linhas_sem_uso"], 1);
    assert!(rep.join("metricas.json").exists());
}

#[test]
fn coletar_conta_achado_antes_e_escape() {
    let t = TempDir::new().unwrap();
    let (cen, rep) = repeticao(t.path(), EXEC_OK);
    gabarito(
        &cen,
        &[
            ("D01", "TASK-01", "execucao.md", "trecho", "defeito-d01"),
            ("D02", "TASK-02", "execucao.md", "trecho", "defeito-d02"),
        ],
    );
    foto(
        &rep,
        "tarefas-inicio",
        "TASK-01",
        "execucao.md",
        "defeito-d01\n",
    );
    foto(&rep, "tarefas-antes", "TASK-01", "execucao.md", "ok\n");
    foto(&rep, "tarefas-fim", "TASK-01", "execucao.md", "ok\n");
    foto(
        &rep,
        "tarefas-inicio",
        "TASK-02",
        "execucao.md",
        "defeito-d02\n",
    );
    foto(
        &rep,
        "tarefas-antes",
        "TASK-02",
        "execucao.md",
        "defeito-d02\n",
    );
    foto(&rep, "tarefas-fim", "TASK-02", "execucao.md", "ok\n");
    registro(&rep, "TASK-02", &["início | queued", "ajuste | feito"]);
    let o = coletar(&rep);
    assert!(o.status.success(), "{}", erro(&o));
    let v = json_de(&o);
    assert_eq!(v["defeitos"], 2);
    assert_eq!(v["achados_antes"], 1);
    assert_eq!(v["escapes"], 1);
    assert_eq!(v["nao_achados"], 0);
}

#[test]
fn coletar_classifica_bordas() {
    let t = TempDir::new().unwrap();
    let (cen, rep) = repeticao(t.path(), EXEC_OK);
    gabarito(
        &cen,
        &[
            ("D03", "TASK-03", "a.md", "trecho", "defeito-d03"),
            ("D04", "TASK-04", "a.md", "trecho", "defeito-d04"),
            ("D05", "TASK-05", "a.md", "correcao", "correcao-d05"),
        ],
    );
    for (tarefa, antes, fim) in [
        ("TASK-03", "defeito-d03\n", "defeito-d03\n"),
        ("TASK-04", "defeito-d04\n", "defeito-d04\n"),
        ("TASK-05", "nada\n", "correcao-d05\n"),
    ] {
        let ini = if tarefa == "TASK-05" { "nada\n" } else { antes };
        foto(&rep, "tarefas-inicio", tarefa, "a.md", ini);
        foto(&rep, "tarefas-antes", tarefa, "a.md", antes);
        foto(&rep, "tarefas-fim", tarefa, "a.md", fim);
    }
    registro(&rep, "TASK-03", &["início | queued", "baseline | coletado"]);
    registro(&rep, "TASK-04", &["início | queued", "testes | failed"]);
    let o = coletar(&rep);
    assert!(o.status.success(), "{}", erro(&o));
    let md = fs::read_to_string(rep.join("defeitos.md")).unwrap();
    assert!(md.contains("| D03 | TASK-03 | nao_achado |"), "{md}");
    assert!(md.contains("| D04 | TASK-04 | escape |"), "{md}");
    assert!(md.contains("| D05 | TASK-05 | escape |"), "{md}");
    let p3 = md.find("D03").unwrap();
    let p4 = md.find("D04").unwrap();
    let p5 = md.find("D05").unwrap();
    assert!(p3 < p4 && p4 < p5);
}

#[test]
fn coletar_recusa_trecho_ausente_na_referencia() {
    let t = TempDir::new().unwrap();
    let (cen, rep) = repeticao(t.path(), EXEC_OK);
    gabarito(&cen, &[("D09", "TASK-09", "a.md", "trecho", "defeito-d09")]);
    foto(&rep, "tarefas-inicio", "TASK-09", "a.md", "limpo\n");
    foto(&rep, "tarefas-antes", "TASK-09", "a.md", "limpo\n");
    foto(&rep, "tarefas-fim", "TASK-09", "a.md", "limpo\n");
    let o = coletar(&rep);
    assert_eq!(o.status.code(), Some(1));
    assert!(
        erro(&o).contains("trecho do gabarito ausente na referência: D09"),
        "{}",
        erro(&o)
    );
}

#[test]
fn coletar_grava_falha_do_executor() {
    let t = TempDir::new().unwrap();
    let (cen, rep) = repeticao(
        t.path(),
        "braco=com\nmodelo=sonnet\nfalhou=true\nmotivo=tempo-limite de 1 s na etapa revisar\n",
    );
    gabarito(&cen, &[("D01", "TASK-01", "a.md", "trecho", "defeito-d01")]);
    foto(&rep, "tarefas-inicio", "TASK-01", "a.md", "defeito-d01\n");
    let o = coletar(&rep);
    assert!(o.status.success(), "{}", erro(&o));
    let v = json_de(&o);
    assert_eq!(v["falhou"], true);
    assert_eq!(v["motivo"], "tempo-limite de 1 s na etapa revisar");
    assert_eq!(v["achados_antes"], 0);
    assert_eq!(v["escapes"], 0);
    let gravado = fs::read_to_string(rep.join("metricas.json")).unwrap();
    assert!(gravado.contains("\"falhou\": true"), "{gravado}");
}

// -------------------------------------------------------------- comparador

#[allow(clippy::too_many_arguments)]
fn metricas(
    cen: &Path,
    braco: &str,
    n: u32,
    modelo: &str,
    tokens: u64,
    tempo: u64,
    achados: u64,
    escapes: u64,
    falhou: bool,
    sem_uso: u64,
) {
    let v = format!(
        "{{\"braco\":\"{braco}\",\"modelo\":\"{modelo}\",\"tokens\":{{\"input\":{tokens},\"output\":0,\"cache_read\":0,\"cache_creation\":0,\"total\":{tokens}}},\"tempo_s\":{tempo},\"defeitos\":3,\"achados_antes\":{achados},\"escapes\":{escapes},\"nao_achados\":0,\"linhas_sem_uso\":{sem_uso},\"falhou\":{falhou},\"motivo\":\"\"}}"
    );
    escrever(
        &cen.join("cen/resultados")
            .join(braco)
            .join(n.to_string())
            .join("metricas.json"),
        &v,
    );
}

fn comparar(raiz: &Path) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_fluxo"))
        .args(["comparar", "cen"])
        .env("FLUXO_BENCHMARKS", raiz)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", erro(&o));
    saida(&o)
}

/// Cenário `cen` dentro de `<tmp>`: `FLUXO_BENCHMARKS` aponta para `<tmp>`.
fn lote(
    t: &Path,
    sem: &[(u64, u64, u64, u64)],
    com: &[(u64, u64, u64, u64)],
    modelo_com: &str,
    com_falha: Option<u32>,
    sem_uso_sem: u64,
) {
    for (i, (tok, tempo, ach, esc)) in sem.iter().enumerate() {
        let su = if i == 0 { sem_uso_sem } else { 0 };
        metricas(
            t,
            "sem",
            i as u32 + 1,
            "sonnet",
            *tok,
            *tempo,
            *ach,
            *esc,
            false,
            su,
        );
    }
    for (i, (tok, tempo, ach, esc)) in com.iter().enumerate() {
        let n = i as u32 + 1;
        metricas(
            t,
            "com",
            n,
            modelo_com,
            *tok,
            *tempo,
            *ach,
            *esc,
            com_falha == Some(n),
            0,
        );
    }
}

fn cinco(tokens: [u64; 5], tempo: u64, ach: u64, esc: u64) -> Vec<(u64, u64, u64, u64)> {
    tokens.iter().map(|t| (*t, tempo, ach, esc)).collect()
}

#[test]
fn comparar_versao_dentro_da_tolerancia() {
    let t = TempDir::new().unwrap();
    lote(
        t.path(),
        &cinco([1000, 1050, 950, 9999, 1100], 100, 0, 3),
        &cinco([1100, 1000, 1150, 1050, 1200], 105, 2, 1),
        "sonnet",
        None,
        0,
    );
    let s = comparar(t.path());
    assert!(s.contains("| Tokens totais | 1050 | 1100 | +4,8% |"), "{s}");
    assert!(s.contains("| Tempo total (s) | 100 | 105 | +5,0% |"), "{s}");
    assert!(s.contains("Decisão: VERSÃO"), "{s}");
}

#[test]
fn comparar_reverter_com_tokens_a_112() {
    let t = TempDir::new().unwrap();
    lote(
        t.path(),
        &cinco([1000; 5], 100, 0, 3),
        &cinco([1120; 5], 100, 2, 1),
        "sonnet",
        None,
        0,
    );
    let s = comparar(t.path());
    assert!(s.contains("+12,0%"), "{s}");
    assert!(s.contains("Decisão: REVERTER"), "{s}");
}

#[test]
fn comparar_inconclusivo_com_quatro_validas() {
    let t = TempDir::new().unwrap();
    lote(
        t.path(),
        &cinco([1000; 5], 100, 0, 3),
        &cinco([1000; 5], 100, 2, 1),
        "sonnet",
        Some(5),
        0,
    );
    let s = comparar(t.path());
    assert!(
        s.contains("Repetições válidas: sem 5 de 5, com 4 de 5"),
        "{s}"
    );
    assert!(s.contains("Decisão: INCONCLUSIVO"), "{s}");
    assert!(
        s.contains("Motivo: menos de 5 repetições válidas (com: 4)"),
        "{s}"
    );
}

#[test]
fn comparar_inconclusivo_com_modelos_diferentes() {
    let t = TempDir::new().unwrap();
    lote(
        t.path(),
        &cinco([1000; 5], 100, 0, 3),
        &cinco([1000; 5], 100, 2, 1),
        "opus",
        None,
        0,
    );
    let s = comparar(t.path());
    assert!(s.contains("Decisão: INCONCLUSIVO"), "{s}");
    assert!(
        s.contains("Motivo: modelos diferentes entre os braços"),
        "{s}"
    );
}

#[test]
fn comparar_empate_reverte() {
    let t = TempDir::new().unwrap();
    lote(
        t.path(),
        &cinco([1000; 5], 100, 1, 2),
        &cinco([1000; 5], 100, 1, 2),
        "sonnet",
        None,
        0,
    );
    let s = comparar(t.path());
    assert!(s.contains("Decisão: REVERTER"), "{s}");
}

#[test]
fn comparar_mostra_linhas_sem_uso() {
    let t = TempDir::new().unwrap();
    lote(
        t.path(),
        &cinco([1000; 5], 100, 0, 3),
        &cinco([1000; 5], 100, 2, 1),
        "sonnet",
        None,
        2,
    );
    let s = comparar(t.path());
    assert!(s.contains("Linhas sem uso: sem 2, com 0"), "{s}");
}

// ----------------------------------------------------------------- executor

fn script() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("executar.sh")
}

fn raiz_repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

const CLAUDE_FALSO: &str = r#"#!/bin/bash
prompt=""; modelo=""; sessao=""; modo=""
while [ $# -gt 0 ]; do
  case "$1" in
    -p) prompt="$2"; shift 2;;
    --model) modelo="$2"; shift 2;;
    --session-id) sessao="$2"; shift 2;;
    --permission-mode) modo="$2"; shift 2;;
    *) shift;;
  esac
done
if [ -n "${FAKE_SLEEP:-}" ]; then sleep "$FAKE_SLEEP"; fi
marcas=$(ls marca-* 2>/dev/null | wc -l | tr -d ' ')
touch "marca-$sessao"
rehearsal=false
case "$(command -v vostok || true)" in *fluxo-bin-*) rehearsal=true;; esac
primeira="${prompt%%$'\n'*}"
echo "$primeira|$modelo|$modo|$marcas|$rehearsal|$(pwd -P)" >> "$FAKE_LOG"
if [ -f vostok/config/main.yml ]; then cp vostok/config/main.yml "$FAKE_MAIN"; fi
if [ "$prompt" = revisar ] && [ "${FAKE_CORRIGE:-0}" = 1 ]; then
  f=vostok/work/tasks/SPEC-900/TASK-01/execucao.md
  sed 's/defeito-d01/corrigido/' "$f" > "$f.tmp" && mv "$f.tmp" "$f"
fi
dir="$CLAUDE_CONFIG_DIR/projects/$(pwd -P | sed 's/[^A-Za-z0-9]/-/g')"
mkdir -p "$dir"
ts=$(date -u +%Y-%m-%dT%H:%M:%S.000Z)
echo "{\"type\":\"assistant\",\"timestamp\":\"$ts\",\"message\":{\"id\":\"m-$sessao\",\"usage\":{\"input_tokens\":1,\"output_tokens\":2,\"cache_read_input_tokens\":3,\"cache_creation_input_tokens\":4}}}" >> "$dir/$sessao.jsonl"
echo '{"type":"result","is_error":false}'
"#;

const VOSTOK_FALSO: &str = "#!/bin/bash\necho \"$*\" >> \"$FAKE_VOSTOK_LOG\"\n";

fn executavel(caminho: &Path, conteudo: &str) {
    use std::os::unix::fs::PermissionsExt;
    escrever(caminho, conteudo);
    fs::set_permissions(caminho, fs::Permissions::from_mode(0o755)).unwrap();
}

struct Ambiente {
    _t: TempDir,
    raiz: PathBuf,
}

impl Ambiente {
    fn novo() -> Self {
        let t = TempDir::new().unwrap();
        let raiz = t.path().to_path_buf();
        fs::create_dir_all(raiz.join("tmp")).unwrap();
        fs::create_dir_all(raiz.join("config")).unwrap();
        executavel(&raiz.join("bin/claude"), CLAUDE_FALSO);
        executavel(&raiz.join("vostok-falso"), VOSTOK_FALSO);
        Ambiente { _t: t, raiz }
    }

    fn benchmarks(&self) -> PathBuf {
        self.raiz.join("benchmarks")
    }

    fn mini(&self) {
        let m = self.benchmarks().join("mini");
        escrever(
            &m.join("projeto-referencia/vostok/config/main.yml"),
            "review:\n  task: false\n",
        );
        escrever(
            &m.join("projeto-referencia/vostok/work/tasks/SPEC-900/TASK-01/execucao.md"),
            "defeito-d01\n",
        );
        escrever(
            &m.join("gabarito.yml"),
            "spec: SPEC-900\ndefeitos:\n  - id: D01\n    tarefa: TASK-01\n    local: Passos\n    categoria: lacuna\n    arquivo: execucao.md\n    trecho: defeito-d01\n",
        );
        escrever(
            &m.join("prompts.md"),
            "introdução ignorada\n\n## revisar\nrevisar\n\n## executar\nexecutar\n\n## finalizar\nfinalizar\n\n",
        );
        escrever(
            &m.join("bracos.yml"),
            &format!(
                "# braços\nsem:\ncom:\n  vostok: {}\n  config:\n    review.task: true\n",
                self.raiz.join("vostok-falso").display()
            ),
        );
    }

    fn comando(&self, args: &[&str]) -> Command {
        let mut c = Command::new("bash");
        c.arg(script())
            .args(args)
            .env(
                "PATH",
                format!("{}:/usr/bin:/bin", self.raiz.join("bin").display()),
            )
            .env("HOME", &self.raiz)
            .env("FLUXO_BENCHMARKS", self.benchmarks())
            .env("CLAUDE_CONFIG_DIR", self.raiz.join("config"))
            .env("TMPDIR", self.raiz.join("tmp"))
            .env("FLUXO_COLETOR", env!("CARGO_BIN_EXE_fluxo"))
            .env("FLUXO_MODELO", "modelo-teste")
            .env("FAKE_LOG", self.raiz.join("claude.log"))
            .env("FAKE_MAIN", self.raiz.join("main-visto.yml"))
            .env("FAKE_VOSTOK_LOG", self.raiz.join("vostok.log"));
        c
    }

    fn log(&self, nome: &str) -> Vec<String> {
        fs::read_to_string(self.raiz.join(nome))
            .unwrap_or_default()
            .lines()
            .map(String::from)
            .collect()
    }

    fn tmp_vazia(&self) -> bool {
        fs::read_dir(self.raiz.join("tmp")).unwrap().count() == 0
    }
}

fn caminho_bash() -> String {
    let o = Command::new("which").arg("bash").output().unwrap();
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

#[test]
fn executor_para_sem_claude() {
    let t = TempDir::new().unwrap();
    let vazia = t.path().join("vazia");
    fs::create_dir_all(&vazia).unwrap();
    let o = Command::new(caminho_bash())
        .arg(script())
        .args(["mini", "com", "1"])
        .env("PATH", &vazia)
        .env("FLUXO_BENCHMARKS", t.path().join("benchmarks"))
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(2), "{}", erro(&o));
    assert!(
        erro(&o).contains("claude não encontrado no PATH"),
        "{}",
        erro(&o)
    );
    assert!(!t.path().join("benchmarks/mini/resultados").exists());
}

#[test]
fn executor_roda_copias_limpas_com_braco() {
    let a = Ambiente::novo();
    a.mini();
    let o = a
        .comando(&["mini", "com", "2"])
        .env("FAKE_CORRIGE", "1")
        .output()
        .unwrap();
    assert!(o.status.success(), "{}\n{}", saida(&o), erro(&o));
    let log = a.log("claude.log");
    assert_eq!(log.len(), 6, "{log:?}");
    let raiz = raiz_repo();
    for rep in 0..2 {
        for (i, etapa) in ["revisar", "executar", "finalizar"].iter().enumerate() {
            let campos: Vec<&str> = log[rep * 3 + i].split('|').collect();
            assert_eq!(campos[0], *etapa);
            assert_eq!(campos[1], "modelo-teste");
            assert_eq!(campos[2], "bypassPermissions");
            assert_eq!(campos[3], i.to_string(), "cópia limpa: {log:?}");
            assert_eq!(campos[4], "true");
            assert!(
                !campos[5].starts_with(raiz.to_str().unwrap()),
                "{}",
                campos[5]
            );
        }
    }
    assert_eq!(
        a.log("vostok.log")
            .iter()
            .filter(|l| *l == "update")
            .count(),
        2
    );
    let m: Value = serde_json::from_str(
        &fs::read_to_string(a.benchmarks().join("mini/resultados/com/1/metricas.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(m["braco"], "com");
    assert_eq!(m["tokens"]["total"], 30);
    assert_eq!(m["achados_antes"], 1);
    assert_eq!(m["falhou"], false);
    assert!(a.tmp_vazia());

    let antes = a.log("vostok.log").len();
    let o = a
        .comando(&["mini", "sem", "1"])
        .env("FAKE_CORRIGE", "1")
        .output()
        .unwrap();
    assert!(o.status.success(), "{}\n{}", saida(&o), erro(&o));
    let log = a.log("claude.log");
    assert_eq!(log.len(), 9);
    assert!(log[6].split('|').nth(4) == Some("false"), "{log:?}");
    assert_eq!(a.log("vostok.log").len(), antes);
}

#[test]
fn executor_registra_tempo_limite() {
    let a = Ambiente::novo();
    a.mini();
    let o = a
        .comando(&["mini", "com", "1"])
        .env("FAKE_SLEEP", "5")
        .env("FLUXO_TEMPO_LIMITE", "1")
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0), "{}", erro(&o));
    assert!(
        saida(&o).contains("repetição 1 de 1: falhou (tempo-limite de 1 s na etapa revisar)"),
        "{}",
        saida(&o)
    );
    let m: Value = serde_json::from_str(
        &fs::read_to_string(a.benchmarks().join("mini/resultados/com/1/metricas.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(m["falhou"], true);
}

/// Valor de `grupo.chave` no `main.yml`: linha `  chave: valor` dentro de `grupo:`.
fn valor_de_config(main: &str, grupo: &str, chave: &str) -> Option<String> {
    let mut dentro = false;
    for l in main.lines() {
        if l.trim_end() == format!("{grupo}:") {
            dentro = true;
        } else if !l.starts_with(' ') && !l.trim().is_empty() && !l.starts_with('#') {
            dentro = false;
        } else if dentro {
            if let Some(v) = l.strip_prefix(&format!("  {chave}:")) {
                return Some(v.trim().to_string());
            }
        }
    }
    None
}

fn copiar(de: &Path, para: &Path) {
    fs::create_dir_all(para).unwrap();
    for e in fs::read_dir(de).unwrap() {
        let e = e.unwrap();
        let destino = para.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            if e.file_name() == "resultados" {
                continue;
            }
            copiar(&e.path(), &destino);
        } else {
            fs::copy(e.path(), destino).unwrap();
        }
    }
}

#[test]
fn executor_cenario_real() {
    let nome = match std::env::var("FLUXO_CENARIO") {
        Ok(n) if !n.is_empty() => n,
        _ => {
            println!("pulado: defina FLUXO_CENARIO");
            return;
        }
    };
    let a = Ambiente::novo();
    let origem = raiz_repo().join("benchmarks").join(&nome);
    copiar(&origem, &a.benchmarks().join(&nome));
    let o = a.comando(&[&nome, "com", "1"]).output().unwrap();
    assert!(o.status.success(), "{}\n{}", saida(&o), erro(&o));
    let rep = a.benchmarks().join(&nome).join("resultados/com/1");
    let m: Value =
        serde_json::from_str(&fs::read_to_string(rep.join("metricas.json")).unwrap()).unwrap();
    assert_eq!(m["falhou"], false, "{m}");

    let prompts = fs::read_to_string(a.benchmarks().join(&nome).join("prompts.md")).unwrap();
    let mut esperadas: Vec<String> = Vec::new();
    let mut proxima = false;
    for l in prompts.lines() {
        if l.starts_with("## ") {
            proxima = true;
        } else if proxima && !l.trim().is_empty() {
            esperadas.push(l.to_string());
            proxima = false;
        }
    }
    let log: Vec<String> = a
        .log("claude.log")
        .iter()
        .map(|l| l.split('|').next().unwrap().to_string())
        .collect();
    assert_eq!(log, esperadas);

    let bracos = fs::read_to_string(a.benchmarks().join(&nome).join("bracos.yml")).unwrap();
    let mut em_com = false;
    let mut tem_vostok = false;
    let main_visto = fs::read_to_string(a.raiz.join("main-visto.yml")).unwrap_or_default();
    for l in bracos.lines() {
        if !l.starts_with(' ') && !l.trim().is_empty() && !l.starts_with('#') {
            em_com = l.trim_end() == "com:";
        } else if em_com {
            if l.starts_with("  vostok:") && !l["  vostok:".len()..].trim().is_empty() {
                tem_vostok = true;
            } else if let Some(r) = l.strip_prefix("    ") {
                if let Some((k, v)) = r.split_once(':') {
                    if let Some((g, c)) = k.split_once('.') {
                        assert_eq!(
                            valor_de_config(&main_visto, g, c).as_deref(),
                            Some(v.trim()),
                            "{k}"
                        );
                    }
                }
            }
        }
    }
    if tem_vostok {
        assert!(rep.join("update.log").exists());
    }
    let gab = fs::read_to_string(a.benchmarks().join(&nome).join("gabarito.yml")).unwrap();
    let n_def = gab
        .lines()
        .filter(|l| l.trim_start().starts_with("- id:"))
        .count();
    assert_eq!(m["defeitos"], n_def);
    assert_eq!(m["nao_achados"], n_def);
    assert!(a.tmp_vazia());
    println!(
        "cenário {nome}: {} etapas, {n_def} defeitos, braço com ok",
        esperadas.len()
    );
}
