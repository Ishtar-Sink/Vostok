//! Comandos `hook`: hooks das ferramentas de agente (SPEC-050).

use std::fs;
use std::io::Read;

use anyhow::Result;
use clap::Subcommand;

use crate::harness::Harness;

/// Regras que precisam valer antes do primeiro comando do agente, relativas à raiz do harness.
const CORE_FILES: [&str; 2] = ["rules/agent/boundaries.md", "rules/project/security.md"];

/// Limite de caracteres que o Claude Code mostra inline na saída de um hook.
const HOOK_LIMIT: usize = 10_000;

#[derive(Debug, Subcommand)]
pub enum HookCommand {
    /// Imprime o núcleo curto do contexto obrigatório (limites, segurança e a ordem de rodar
    /// `vostok context load`) para o hook de início de sessão.
    SessionStart,
    /// Lê o JSON do hook PreToolUse no stdin e recusa `vostok context load` com a saída
    /// redirecionada, num pipe ou capturada.
    PreBash,
}

pub fn run(command: HookCommand) -> Result<()> {
    match command {
        HookCommand::SessionStart => {
            let mut text = core(&Harness::from_current_dir()?);
            let count = text.chars().count();
            if count >= HOOK_LIMIT {
                let warning = format!(
                    "Aviso: o núcleo tem {count} caracteres, acima do limite de 10.000 do hook; \
                     rode `vostok context load` agora."
                );
                eprintln!("{warning}");
                text.push_str(&format!("\n{warning}\n"));
            }
            print!("{text}");
            Ok(())
        }
        HookCommand::PreBash => {
            let mut input = String::new();
            std::io::stdin().read_to_string(&mut input)?;
            if let Some(discard) = bash_command(&input).and_then(|command| discarded_load(&command))
            {
                println!("{}", deny(discard));
            }
            Ok(())
        }
    }
}

/// Comando de shell do PreToolUse do Bash; qualquer outra entrada é ignorada sem erro, para que
/// o hook nunca bloqueie a ferramenta por uma entrada que não entende.
fn bash_command(input: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(input).ok()?;
    if value["tool_name"] != "Bash" {
        return None;
    }
    value["tool_input"]["command"].as_str().map(str::to_owned)
}

fn deny(discard: Discard) -> serde_json::Value {
    let reason = format!(
        "Recusado pelo harness: `vostok context load` com a saída {}. Execute-o sem \
         redirecionamento, pipe ou captura e leia a saída inteira (SPEC-050 RF-03).",
        discard.reason()
    );
    serde_json::json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason
        }
    })
}

fn core(harness: &Harness) -> String {
    let mut text = format!(
        "# Núcleo do contexto obrigatório do harness\n\nRaiz do harness: `{}`\n",
        harness.root.display()
    );
    for item in CORE_FILES {
        let Ok(content) = fs::read_to_string(harness.root.join(item)) else {
            continue;
        };
        text.push_str(&format!("\n--- arquivo: {item} ---\n\n{content}"));
        if !content.ends_with('\n') {
            text.push('\n');
        }
    }
    text.push_str(
        "\n## Próximo passo obrigatório\n\nRode agora `vostok context load` e leia a saída \
         inteira, sem redirecionar, filtrar nem capturar: ela traz as demais regras, os templates \
         de resposta e a persona. Rode de novo depois de cada compactação ou limpeza da conversa.\n",
    );
    text
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Discard {
    Redirected,
    Piped,
    Captured,
}

impl Discard {
    fn reason(self) -> &'static str {
        match self {
            Discard::Redirected => "redirecionada",
            Discard::Piped => "num pipe",
            Discard::Captured => "capturada",
        }
    }
}

/// Indica se `command` roda `vostok context load` com a saída descartada para o agente.
fn discarded_load(command: &str) -> Option<Discard> {
    Scanner::new(command).run()
}

/// Abertura de shell que muda o contexto dos caracteres seguintes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Open {
    Top,
    Substitution,
    Backtick,
    ProcessIn,
    ProcessOut,
    Subshell,
    Group,
    DoubleQuote,
}

impl Open {
    fn captures(self) -> bool {
        matches!(
            self,
            Open::Substitution | Open::Backtick | Open::ProcessIn | Open::ProcessOut
        )
    }

    fn closed_by_paren(self) -> bool {
        matches!(
            self,
            Open::Substitution | Open::ProcessIn | Open::ProcessOut | Open::Subshell
        )
    }
}

/// Palavras de um nível da pilha: a palavra em leitura e as últimas do comando simples.
struct Frame {
    open: Open,
    word: String,
    word_started: bool,
    quoted: bool,
    recent: Vec<String>,
}

impl Frame {
    fn new(open: Open) -> Self {
        Self {
            open,
            word: String::new(),
            word_started: false,
            quoted: false,
            recent: Vec::new(),
        }
    }

    fn push_char(&mut self, c: char) {
        self.word.push(c);
        self.word_started = true;
    }

    /// Marca a palavra atual como tendo texto entre aspas, escapado ou substituído, que não
    /// conta como a palavra literal `vostok`, `context` ou `load`.
    fn quote_word(&mut self) {
        self.word_started = true;
        self.quoted = true;
    }

    /// Fecha a palavra atual e diz se ela completou `vostok context load`.
    fn finish_word(&mut self) -> bool {
        if !self.word_started {
            return false;
        }
        let word = std::mem::take(&mut self.word);
        self.word_started = false;
        if std::mem::take(&mut self.quoted) {
            self.recent.clear();
            return false;
        }
        self.recent.push(word);
        if self.recent.len() > 3 {
            self.recent.remove(0);
        }
        is_load(&self.recent)
    }

    fn end_command(&mut self) -> bool {
        let found = self.finish_word();
        self.recent.clear();
        found
    }

    /// Descarta a palavra atual quando ela é só o número do descritor de um redirecionamento.
    fn take_descriptor(&mut self) -> Option<String> {
        let is_descriptor = self.word_started
            && !self.quoted
            && !self.word.is_empty()
            && self.word.chars().all(|c| c.is_ascii_digit());
        if !is_descriptor {
            return None;
        }
        self.word_started = false;
        Some(std::mem::take(&mut self.word))
    }
}

fn is_load(words: &[String]) -> bool {
    matches!(words, [vostok, context, load]
        if (vostok == "vostok" || vostok.ends_with("/vostok"))
            && context == "context"
            && load == "load")
}

/// O que um trecho de primeiro nível do comando faz com uma carga do contexto.
#[derive(Default)]
struct Segment {
    load: bool,
    captured: bool,
    piped: bool,
    redirected: bool,
}

impl Segment {
    fn verdict(&self) -> Option<Discard> {
        if !self.load {
            None
        } else if self.captured {
            Some(Discard::Captured)
        } else if self.piped {
            Some(Discard::Piped)
        } else if self.redirected {
            Some(Discard::Redirected)
        } else {
            None
        }
    }
}

/// Leitor de caracteres do comando de shell, com aspas, escapes e uma pilha de aberturas.
struct Scanner {
    chars: Vec<char>,
    pos: usize,
    frames: Vec<Frame>,
    segment: Segment,
}

impl Scanner {
    fn new(command: &str) -> Self {
        Self {
            chars: command.chars().collect(),
            pos: 0,
            frames: vec![Frame::new(Open::Top)],
            segment: Segment::default(),
        }
    }

    fn run(mut self) -> Option<Discard> {
        while let Some(c) = self.next() {
            let verdict = if self.top().open == Open::DoubleQuote {
                self.double_quoted(c);
                None
            } else {
                self.code(c)
            };
            if verdict.is_some() {
                return verdict;
            }
        }
        while self.frames.len() > 1 {
            self.end_command();
            self.frames.pop();
        }
        self.end_command();
        self.segment.verdict()
    }

    fn next(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        self.pos += 1;
        c
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn eat(&mut self, expected: char) -> bool {
        let matched = self.peek() == Some(expected);
        if matched {
            self.pos += 1;
        }
        matched
    }

    fn top(&mut self) -> &mut Frame {
        self.frames
            .last_mut()
            .expect("a pilha sempre tem o primeiro nível")
    }

    fn note(&mut self, found: bool) {
        if found {
            self.segment.load = true;
            self.segment.captured |= self.frames.iter().any(|frame| frame.open.captures());
        }
    }

    fn finish_word(&mut self) {
        let found = self.top().finish_word();
        self.note(found);
    }

    fn end_command(&mut self) {
        let found = self.top().end_command();
        self.note(found);
    }

    fn push(&mut self, open: Open) {
        self.top().quote_word();
        self.frames.push(Frame::new(open));
    }

    fn pop(&mut self) {
        self.end_command();
        self.frames.pop();
    }

    /// Fecha um comando; no primeiro nível, fecha também o trecho e devolve o veredito dele.
    fn separator(&mut self) -> Option<Discard> {
        self.end_command();
        if self.frames.len() > 1 {
            return None;
        }
        std::mem::take(&mut self.segment).verdict()
    }

    fn at_boundary(&self) -> bool {
        self.peek()
            .is_none_or(|c| c.is_whitespace() || ";|&<>()".contains(c))
    }

    fn code(&mut self, c: char) -> Option<Discard> {
        match c {
            '\n' | ';' => return self.separator(),
            '&' => return self.ampersand(),
            '|' => return self.bar(),
            '>' => self.greater(),
            '<' => {
                if self.eat('(') {
                    self.push(Open::ProcessIn);
                } else {
                    self.finish_word();
                }
            }
            '(' => {
                self.finish_word();
                self.frames.push(Frame::new(Open::Subshell));
            }
            ')' if self.top().open.closed_by_paren() => self.pop(),
            '`' if self.top().open == Open::Backtick => self.pop(),
            '`' => self.push(Open::Backtick),
            '$' if self.eat('(') => self.push(Open::Substitution),
            '"' => self.push(Open::DoubleQuote),
            '\'' => self.single_quoted(),
            '\\' => self.escaped(),
            '{' if !self.top().word_started && self.at_boundary() => {
                self.end_command();
                self.frames.push(Frame::new(Open::Group));
            }
            '}' if !self.top().word_started
                && self.top().open == Open::Group
                && self.at_boundary() =>
            {
                self.pop()
            }
            '#' if !self.top().word_started => self.comment(),
            c if c.is_whitespace() || c == ')' => self.finish_word(),
            c => self.top().push_char(c),
        }
        None
    }

    fn double_quoted(&mut self, c: char) {
        match c {
            '"' => {
                self.frames.pop();
            }
            '\\' => {
                self.next();
            }
            '$' if self.eat('(') => self.frames.push(Frame::new(Open::Substitution)),
            '`' => self.frames.push(Frame::new(Open::Backtick)),
            _ => {}
        }
    }

    fn ampersand(&mut self) -> Option<Discard> {
        if self.eat('&') {
            return self.separator();
        }
        if self.eat('>') {
            self.finish_word();
            self.eat('>');
            self.segment.redirected = true;
            return None;
        }
        self.separator()
    }

    fn bar(&mut self) -> Option<Discard> {
        if self.eat('|') {
            return self.separator();
        }
        self.eat('&');
        self.segment.piped = true;
        self.end_command();
        None
    }

    fn greater(&mut self) {
        if self.eat('(') {
            self.push(Open::ProcessOut);
            return;
        }
        let descriptor = self.top().take_descriptor();
        if descriptor.is_none() {
            self.finish_word();
        }
        if self.eat('&') {
            let mut target = String::new();
            while let Some(c) = self.peek().filter(|c| c.is_ascii_digit() || *c == '-') {
                target.push(c);
                self.pos += 1;
            }
            let joins_stderr = descriptor.as_deref() == Some("2") && target == "1";
            if !(joins_stderr && self.at_boundary()) {
                self.segment.redirected = true;
            }
            return;
        }
        if !self.eat('>') {
            self.eat('|');
        }
        self.segment.redirected = true;
    }

    fn single_quoted(&mut self) {
        self.top().quote_word();
        while let Some(c) = self.next() {
            if c == '\'' {
                break;
            }
        }
    }

    fn escaped(&mut self) {
        if self.eat('\n') {
            return;
        }
        self.top().quote_word();
        self.next();
    }

    fn comment(&mut self) {
        while self.peek().is_some_and(|c| c != '\n') {
            self.pos += 1;
        }
    }
}
