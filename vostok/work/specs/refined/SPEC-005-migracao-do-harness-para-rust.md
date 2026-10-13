---
id: SPEC-005
title: "Migração do Harness para Rust"
status: planned
author: "Victor Batalha"
created_at: 2026-10-11T09:15:16Z
updated_at: 2026-10-13T11:16:36Z
tags: []
type: feature
source: "draft/SPEC-005-migracao-do-harness-para-rust.md"
refined_at: 2026-10-13T11:02:40Z
adjustments: 1
adjusted_at: 2026-10-13T11:10:30Z
content_hash: b392a49c2e311dedc0407449b6b1117ebf159180867fe0c1fc779c45a4412d14
approved_at: 2026-10-13T11:10:35Z
approved_hash: b392a49c2e311dedc0407449b6b1117ebf159180867fe0c1fc779c45a4412d14
---

# Migração do Harness para Rust - Especificação Refinada

Substitui os scripts bash do harness pela CLI `vostok`, em Rust: um binário que empacota o harness, o instala nos projetos com uma inicialização interativa, atualiza o que vem dele sem tocar no que é do projeto e executa todos os comandos que hoje são scripts. Este projeto passa a usar o harness instalado.

## Objetivo

O harness em bash ficou grande (cerca de 6.400 linhas, mais 1.800 da suíte, que leva cerca de 95 s) e só funciona dentro deste repositório. O resultado esperado é a versão 0.1.0 do binário `vostok` para Linux e macOS, publicada no GitHub Releases, que instala o harness em qualquer projeto, executa todos os fluxos atuais e substitui por completo os scripts bash e a suíte bash.

## Escopo

### Dentro do escopo

- Crate Rust em `apps/cli`, binário `vostok`, versão 0.1.0.
- Reescrita de todos os scripts de `packages/harness/scripts/default/`, de `packages/harness/scripts/lib/` e de `packages/harness/skills/default/*/scripts/` como subcomandos do `vostok`, com o mesmo comportamento (resposta 1A).
- Conteúdo do harness (skills, templates, regras, configurações padrão, contexto) empacotado no binário a partir de `packages/harness`.
- `vostok init`: inicialização interativa, com `--yes`, que instala o harness em `vostok/` na raiz do projeto e gera os arquivos de roteamento.
- `vostok update`: regrava o que vem do binário e preserva o que é do projeto (resposta 3A).
- SKILL.md, `AGENTS.md` e redirecionamentos chamando o `vostok` em vez dos scripts.
- Testes em Rust (`cargo test`) cobrindo o comportamento da suíte bash, que é aposentada (resposta 5A).
- Publicação da 0.1.0 no GitHub Releases, com binários para Linux e macOS (x86_64 e arm64) e um script de instalação (resposta 4A).
- Migração deste projeto para o harness instalado em `vostok/`, com `packages/harness` só como fonte (architecture-08).
- Remoção dos scripts bash e da suíte bash depois da migração.

### Fora do escopo

- Windows.
- Pacote npm e `cargo install` como canais oficiais.
- Servidor HTTP, MCP e painel web (fase 2).
- Mudanças de comportamento nos fluxos: os subcomandos reproduzem o que os scripts fazem hoje.
- Mudança nos formatos de arquivo (front-matter, YAML, templates, IDs, tabelas).
- Escolha da pasta de instalação na inicialização: a pasta é fixa, `vostok/` (resposta 2C).

## Requisitos funcionais

| ID | Requisito |
|---|---|
| RF-01 | O sistema deve ter um crate Rust em `apps/cli` que gera o binário `vostok`, versão 0.1.0. |
| RF-02 | O binário deve empacotar, em tempo de compilação, o conteúdo distribuível de `packages/harness`: `skills/default`, `templates`, `rules`, `context`, `config` e `memory` sem entradas. |
| RF-03 | O `vostok init` deve instalar o harness em `vostok/`, na raiz do projeto. |
| RF-04 | O `vostok init` deve perguntar, em modo interativo, os caminhos de `config/main.yml` que apontam para fora do harness (`docs`, `spec_docs`) e, numa seleção múltipla, quais ferramentas o projeto usa entre as suportadas (Claude Code, Codex, Cursor). |
| RF-05 | O `vostok init --yes` deve instalar com os valores padrão, sem perguntas, e aceitar `--tools <lista>` para escolher as ferramentas; sem `--tools`, instala para todas as suportadas. |
| RF-06 | O `vostok init` deve gerar o `AGENTS.md` e, só para as ferramentas escolhidas, os arquivos de roteamento delas (`CLAUDE.md` só com Claude Code) e os redirecionamentos de skills nas pastas delas, apontando para `vostok/`. |
| RF-07 | O `vostok init` deve recusar um projeto que já tem `vostok/` e indicar o `vostok update`. |
| RF-08 | O `vostok init` deve gravar em `vostok/.manifest` a versão do binário e o hash de cada arquivo vindo do binário. |
| RF-09 | O `vostok update` deve regravar os arquivos vindos do binário e preservar configurações, regras de projeto, skills customizadas, memória, `work/` e `.state/`. |
| RF-10 | O `vostok update` deve, antes de regravar um arquivo vindo do binário que foi alterado no projeto, guardar a versão local como `<arquivo>.local` e listá-la na saída. |
| RF-11 | O `vostok update` deve acrescentar ao `config/context.yml` e às configurações do projeto as chaves novas da versão, sem alterar as existentes. |
| RF-12 | O `vostok update` deve regenerar os redirecionamentos e o catálogo de skills. |
| RF-13 | O `vostok` deve ter um subcomando para cada script atual, com os mesmos argumentos, mensagens e códigos de saída (tabela em "Contratos e interfaces"). |
| RF-14 | Os subcomandos devem localizar o harness procurando `vostok/config/main.yml` a partir do diretório atual, subindo até a raiz. |
| RF-15 | Os subcomandos devem ler caminhos, status, tipos, seções e sequências de `vostok/config/*.yml`, sem valores fixos no código. |
| RF-16 | O `vostok finalize` deve gerar os mesmos documentos HTML que o `finalize-spec.sh` gera hoje. |
| RF-17 | Os SKILL.md padrão, o template de skill e o template de redirecionamento devem chamar o `vostok` em vez dos scripts. |
| RF-18 | O `vostok skill create` deve gerar o SKILL.md de uma skill nova já com as chamadas ao `vostok`. |
| RF-19 | O `AGENTS.md` gerado deve mandar executar `vostok context load` para o contexto obrigatório. |
| RF-20 | O repositório deve ter um workflow do GitHub Actions que, numa tag `v*`, compila os binários de Linux e macOS (x86_64 e arm64) e os publica no GitHub Releases. |
| RF-21 | O repositório deve ter um script `install.sh` que detecta o sistema e a arquitetura, baixa o binário da release e o instala no `PATH` do usuário. |
| RF-22 | Este projeto deve passar a usar o harness em `vostok/`, com `work/`, `memory/`, `.state/`, as regras de projeto e o mapa de contexto movidos de `packages/harness`. |
| RF-23 | Os scripts bash e a suíte bash devem ser removidos depois que este projeto estiver usando o binário. |
| RF-24 | O `vostok --version` deve imprimir a versão do binário. |
| RF-25 | A escolha de ferramentas deve ficar em `vostok/config/skills.yml` (`redirect_dirs`), e o `vostok update` e o `vostok skill finalize` devem gerar redirecionamentos só para as ferramentas registradas. |

## Requisitos não funcionais

| ID | Requisito | Como verificar |
|---|---|---|
| RNF-01 | O binário roda em Linux e macOS, x86_64 e arm64, sem dependências em tempo de execução além do sistema. | Release com os quatro binários; `vostok --version` no Linux (WSL) e na Mac de testes. |
| RNF-02 | Os testes Rust rodam mais rápido que a suíte bash atual (cerca de 95 s). | `time cargo test` no Linux, comparado com a medição da suíte bash. |
| RNF-03 | `cargo clippy -- -D warnings` e `cargo fmt --check` passam (code-style-04). | Execução no workflow e na verificação das tarefas. |
| RNF-04 | O binário não acessa a rede, exceto o `install.sh`, que só baixa a release. | Revisão das dependências e do código: nenhum cliente HTTP no crate. |
| RNF-05 | Os subcomandos não leem `.env` nem arquivos de credenciais (security-01). | Revisão e teste: nenhum caminho de `.env` lido pelo binário. |
| RNF-06 | Mensagens ao usuário em pt-br, com o mesmo texto dos scripts atuais. | Testes de integração conferem as mensagens. |
| RNF-07 | Formatos de arquivo idênticos aos dos scripts (DEC-002). | Testes que comparam a saída do binário com fixtures geradas pelos scripts atuais. |

## Regras de negócio

| ID | Regra |
|---|---|
| RN-01 | Arquivos vindos do binário: `skills/default`, `templates`, `rules/agent`, `context` e os templates de configuração. Arquivos do projeto: `config/*.yml` depois de instalados, `rules/project`, `skills/custom`, `memory`, `work` e `.state`. |
| RN-02 | O `vostok update` nunca apaga nem sobrescreve sem cópia um arquivo do projeto. |
| RN-03 | Um subcomando que altera estado (status, hash, aprovação, commit) segue as mesmas recusas dos scripts atuais. |
| RN-04 | Commits continuam só pelo `vostok commit` a pedido do usuário (agent/boundaries-01). |
| RN-05 | A pasta de instalação é sempre `vostok/` na raiz do projeto. |

## Fluxos

### Fluxo principal

1. O usuário instala o binário com o `install.sh` da release.
2. Na raiz do projeto, o usuário executa `vostok init`.
3. O binário pergunta os caminhos de documentos e as ferramentas que recebem redirecionamentos.
4. O binário cria `vostok/` com o conteúdo empacotado, grava as respostas em `vostok/config/main.yml` e `vostok/config/skills.yml` e grava o `vostok/.manifest`.
5. O binário gera `AGENTS.md`, `CLAUDE.md`, os redirecionamentos e o catálogo de skills.
6. O agente da ferramenta lê o `AGENTS.md`, executa `vostok context load` e segue as skills, que chamam o `vostok`.

### Fluxos alternativos

- FA-01 (passo 2): com `--yes`, o passo 3 é pulado e os valores padrão são usados.
- FA-02 (passo 2): o projeto já tem `vostok/`. O `init` recusa e indica o `vostok update`.
- FA-03 (fora do init): uma versão nova do binário. O usuário executa `vostok update`, que regrava os arquivos vindos do binário, guarda as versões locais alteradas como `.local`, acrescenta as chaves novas de configuração e regenera redirecionamentos e catálogo.
- FA-04 (passo 1): sistema ou arquitetura sem binário publicado. O `install.sh` termina com erro e lista as plataformas disponíveis.
- FA-05 (este projeto): a migração move `work/`, `memory/`, `.state/`, as regras de projeto e o mapa para `vostok/`, gera o roteamento e remove os scripts e a suíte bash.

## Casos de borda

| ID | Situação | Comportamento esperado | Requisitos |
|---|---|---|---|
| CB-01 | `vostok init` com `AGENTS.md` ou `CLAUDE.md` já existentes e sem relação com o harness. | Guarda a versão anterior como `<arquivo>.local`, gera o novo e lista o arquivo guardado. | RF-06 |
| CB-02 | Subcomando executado fora de um projeto com `vostok/`. | Erro que indica o `vostok init`, com saída 1. | RF-14 |
| CB-03 | `vostok update` com um arquivo vindo do binário apagado no projeto. | O arquivo é recriado, sem `.local`. | RF-09, RF-10 |
| CB-04 | `vostok update` com o mesmo binário já instalado. | Nenhum arquivo muda, e a saída informa que já está atualizado. | RF-09 |
| CB-05 | `vostok update` num projeto com skill customizada de mesmo nome que uma skill padrão nova. | A customizada é preservada, e a saída avisa do conflito. | RF-09, RF-12 |
| CB-06 | Execução interativa sem terminal (stdin não é TTY) e sem `--yes`. | Erro que pede `--yes`, com saída 1. | RF-04, RF-05 |
| CB-07 | Configuração com chave obrigatória ausente. | Erro que cita o arquivo e a chave, como nos scripts atuais. | RF-15 |
| CB-08 | Nenhuma ferramenta escolhida na inicialização, ou `--tools` com ferramenta não suportada. | Erro que lista as ferramentas suportadas, sem instalar nada. | RF-04, RF-05 |

## Contratos e interfaces

Comandos novos:

```bash
vostok --version
vostok init [--yes] [--tools claude-code,codex,cursor]
vostok update
```

Ferramentas suportadas e o que cada uma recebe:

```text
claude-code  -> CLAUDE.md e .claude/skills/<skill>/SKILL.md
codex        -> .agents/skills/<skill>/SKILL.md
cursor       -> .cursor/skills/<skill>/SKILL.md
(todas)      -> AGENTS.md
```

Correspondência entre os scripts atuais e os subcomandos. As opções (`--validate`, `--approve`, `--record-manual`...) passam a ser subcomandos com o mesmo nome, e os argumentos continuam na mesma ordem:

```text
scripts/default/context/load-context.sh            -> vostok context load
scripts/default/context/load-extra-context.sh      -> vostok context load <caminho...> | --skill <nome>
scripts/default/git/check-commits.sh               -> vostok commit check
skills/default/approve-commit/.../approve-commit.sh -> vostok commit show|approve|revert|commit
scripts/default/memory/add-memory.sh               -> vostok memory add
scripts/default/memory/search-memory.sh            -> vostok memory search
scripts/default/memory/update-memory.sh            -> vostok memory update
scripts/default/rules/validate-rules.sh            -> vostok rules validate
skills/default/init-harness/.../init-harness.sh    -> vostok rules detect|write|preview|load
skills/default/adjust-rules/.../adjust-rules.sh    -> vostok rules next-id|remove|create-category|load|unload
scripts/default/specs/create-new-spec.sh           -> vostok spec new
scripts/default/specs/list-specs.sh                -> vostok spec list
scripts/default/specs/create-refined-spec.sh       -> vostok spec refine|validate|mark-refined
skills/default/adjust-spec/.../adjust-spec.sh      -> vostok spec adjust ...
skills/default/approve-spec/.../approve-spec.sh    -> vostok spec approve ...
skills/default/finalize-spec/.../finalize-spec.sh  -> vostok finalize sources|create|validate|build|archive
skills/default/plan-tasks/.../plan-tasks.sh        -> vostok plan create|score|validate
skills/default/adjust-plan/.../adjust-plan.sh      -> vostok plan adjust ...
skills/default/approve-plan/.../approve-plan.sh    -> vostok plan approve ...
scripts/default/tasks/load-task.sh                 -> vostok task load
scripts/default/tasks/check-done.sh                -> vostok task check-done
skills/default/approve-task/.../approve-task.sh    -> vostok task approve ...
skills/default/adjust-task/.../adjust-task.sh      -> vostok task adjust ...
skills/default/execute-task/.../execute-task.sh    -> vostok task start|baseline
skills/default/finish-task/.../finish-task.sh      -> vostok task tests|regression|evidence|finish|resume
skills/default/create-skill/.../create-skill.sh    -> vostok skill create|script|finalize
```

Estrutura instalada:

```text
vostok/
  .manifest          # versão e hash de cada arquivo vindo do binário
  config/  context/  memory/  rules/agent/  rules/project/
  skills/default/  skills/custom/  templates/  work/  .state/
```

Formato do `vostok/.manifest`:

```yaml
version: 0.1.0
files:
  skills/default/refine-spec/SKILL.md: <sha256>
```

## Componentes afetados

| Caminho | Ação | Motivo |
|---|---|---|
| `apps/cli` | criar | Crate Rust do binário `vostok` (RF-01 a RF-24). |
| `.github/workflows/release.yml` | criar | Build e publicação da release (RF-20). |
| `install.sh` | criar | Instalação a partir da release (RF-21). |
| `vostok` | criar | Harness instalado neste projeto (RF-22). |
| `packages/harness/skills` | alterar | SKILL.md chamando o `vostok` (RF-17). |
| `packages/harness/templates/skill/skill.md` | alterar | Template de skill chamando o `vostok` (RF-17, RF-18). |
| `packages/harness/templates/skill/redirect.md` | alterar | Redirecionamento apontando para `vostok/` (RF-06). |
| `packages/harness/config/main.yml` | alterar | Caminhos relativos a `vostok/` (RF-03). |
| `packages/harness/scripts` | remover | Scripts substituídos pelo binário (RF-23). |
| `packages/harness/tests` | remover | Suíte bash substituída pelo `cargo test` (RF-23). |
| `packages/harness/work` | remover | Movido para `vostok/work` (RF-22). |
| `packages/harness/memory` | remover | Movido para `vostok/memory` (RF-22). |
| `AGENTS.md` | alterar | Gerado pelo `vostok init`, com `vostok context load` (RF-19). |
| `CLAUDE.md` | alterar | Gerado pelo `vostok init` (RF-06). |
| `.claude/skills` | alterar | Redirecionamentos para `vostok/` (RF-06). |
| `.agents/skills` | alterar | Redirecionamentos para `vostok/` (RF-06). |
| `README.md` | alterar | Instalação e uso pelo binário. |

## Premissas e dependências

- O repositório será publicado no GitHub, na organização Ishtar Sink, antes da release. Hoje não há remoto configurado; criar o repositório e publicar a tag dependem de autorização do usuário.
- A Mac de testes (arm64) tem `cargo` e `rustc` instalados, o que resolve a pendência da DEC-002.
- O Linux de desenvolvimento (WSL) tem `cargo` e `rustc` instalados.
- Os formatos atuais (front-matter, YAML, templates) são a referência de compatibilidade (DEC-002, RNF-07).
- Os runners do GitHub Actions compilam para macOS x86_64 e arm64 e para Linux x86_64 e arm64.

## Riscos

| Risco | Impacto | Mitigação |
|---|---|---|
| Diferenças de comportamento entre scripts e binário quebram fluxos das skills. | alto | Fixtures geradas pelos scripts atuais comparadas com a saída do binário (RNF-07) e cenários de agente rodados de novo nos fluxos afetados. |
| Reescrever cerca de 6.400 linhas de bash é uma entrega grande, com tarefas longas. | alto | Tarefas por grupo de comandos (contexto, memória, regras, specs, planos, tarefas, finalização, commits, skills), cada uma com testes próprios. |
| O `vostok update` sobrescreve uma personalização do usuário. | médio | Cópia `.local` de todo arquivo vindo do binário alterado no projeto (RF-10, RN-02). |
| A migração deste projeto perde `work/` ou `memory/`. | alto | Mover com conferência de contagem e hash antes de remover a origem; commit separado. |
| A geração de HTML do finalize difere da atual. | médio | Comparação dos documentos gerados para uma spec arquivada (SPEC-006) antes e depois. |
| O workflow de release falha numa plataforma. | baixo | Build das quatro plataformas em toda tag, com a release publicada só se todas passarem. |

## Casos de teste

| ID | Cenário | Passos | Resultado esperado | Requisitos |
|---|---|---|---|---|
| CT-01 | Versão. | `cargo build` e `vostok --version`. | Imprime `0.1.0`. | RF-01, RF-24 |
| CT-02 | Conteúdo empacotado. | Teste que lista os arquivos embutidos. | Contém skills padrão, templates, regras, contexto, configurações e memória sem entradas; não contém `work/`, `.state/` nem `tests/`. | RF-02 |
| CT-03 | Inicialização padrão. | `vostok init --yes` numa pasta temporária. | `vostok/` criada com o conteúdo, `AGENTS.md`, `CLAUDE.md`, redirecionamentos e `.manifest` com versão e hashes. | RF-03, RF-05, RF-06, RF-08 |
| CT-04 | Inicialização interativa. | `vostok init` com respostas simuladas por stdin, escolhendo só Codex. | Caminhos e ferramentas gravados em `main.yml` e `skills.yml`; existem `AGENTS.md` e `.agents/skills`; não existem `CLAUDE.md`, `.claude/skills` nem `.cursor/skills`. | RF-04, RF-06 |
| CT-05 | Inicialização repetida. | `vostok init --yes` duas vezes. | A segunda recusa e indica o `vostok update`. | RF-07 |
| CT-06 | Atualização preserva o projeto. | Instalar, alterar regra de projeto, memória, `work/` e uma skill customizada; executar `vostok update`. | Nada disso muda. | RF-09 |
| CT-07 | Atualização com arquivo do binário alterado. | Alterar um SKILL.md padrão e executar `vostok update`. | A versão local fica em `.local`, listada na saída, e o arquivo volta ao do binário. | RF-10 |
| CT-08 | Chaves novas de configuração. | Remover uma chave de um `config/*.yml` instalado e executar `vostok update`. | A chave volta; as demais ficam como estavam. | RF-11 |
| CT-09 | Regeneração no update. | Apagar um redirecionamento e executar `vostok update`. | Redirecionamento e catálogo regenerados. | RF-12 |
| CT-10 | Equivalência dos subcomandos. | Para cada script, rodar o mesmo cenário dos casos da suíte bash com o subcomando correspondente. | Mesmas mensagens, códigos de saída e arquivos. | RF-13, RF-15 |
| CT-11 | Localização do harness. | Executar um subcomando a partir de uma subpasta do projeto e de fora dele. | Na subpasta funciona; fora, erro que indica o `vostok init`. | RF-14 |
| CT-12 | Finalização equivalente. | Gerar os documentos de uma spec de fixture pelo script atual e pelo binário. | HTML com o mesmo conteúdo. | RF-16 |
| CT-13 | Skills chamam o binário. | Busca por `.sh` e por `scripts/` nos SKILL.md padrão e nos templates de skill. | Nenhuma ocorrência; as chamadas são ao `vostok`. | RF-17, RF-18 |
| CT-14 | AGENTS.md gerado. | Ler o `AGENTS.md` do CT-03. | Manda executar `vostok context load`. | RF-19 |
| CT-15 | Workflow de release. | Validar o workflow e executar o build das quatro plataformas num tag de teste. | Quatro binários gerados e anexados à release. | RF-20 |
| CT-16 | Instalação. | Executar o `install.sh` no Linux e na Mac de testes apontando para a release. | `vostok --version` funciona nas duas; plataforma sem binário recebe erro com a lista disponível. | RF-21 |
| CT-17 | Este projeto migrado. | Conferir `vostok/` deste repositório e contar specs, tarefas e entradas de memória antes e depois. | Mesmas contagens e hashes; skills funcionando pelo binário. | RF-22 |
| CT-19 | Ferramentas no init não interativo e no update. | `vostok init --yes --tools claude-code`; criar uma skill customizada e executar `vostok skill finalize` e `vostok update`; depois `--tools` com ferramenta inválida e com lista vazia. | Só `CLAUDE.md` e `.claude/skills` recebem redirecionamentos, também da skill nova e no update; os casos inválidos falham listando as suportadas. | RF-05, RF-25 |
| CT-18 | Bash removido. | Procurar arquivos `.sh` em `packages/harness`. | Nenhum, exceto os que o binário empacota de propósito (se houver). | RF-23 |

## Critérios de aceite

- [ ] CA-01: o binário `vostok` 0.1.0 compila e imprime a versão (RF-01, RF-24).
- [ ] CA-02: o binário contém o conteúdo distribuível do harness e nada de `work/`, `.state/` ou testes (RF-02).
- [ ] CA-03: o `vostok init` instala em `vostok/`, pergunta caminhos e ferramentas, aceita `--yes`, gera o roteamento e o manifesto e recusa projeto já instalado (RF-03, RF-04, RF-05, RF-06, RF-07, RF-08).
- [ ] CA-11: só as ferramentas escolhidas recebem roteamento e redirecionamentos, no init, na criação de skills e no update (RF-04, RF-05, RF-06, RF-25).
- [ ] CA-04: o `vostok update` preserva o projeto, guarda versões locais como `.local`, acrescenta chaves novas e regenera redirecionamentos e catálogo (RF-09, RF-10, RF-11, RF-12).
- [ ] CA-05: cada script atual tem um subcomando com o mesmo comportamento, conferido pelos testes Rust (RF-13, RF-14, RF-15, RF-16).
- [ ] CA-06: SKILL.md, templates e `AGENTS.md` chamam o `vostok` (RF-17, RF-18, RF-19).
- [ ] CA-07: a tag `v0.1.0` publica no GitHub Releases os binários de Linux e macOS, x86_64 e arm64, e o `install.sh` os instala (RF-20, RF-21).
- [ ] CA-08: este projeto usa o harness em `vostok/`, sem perda de specs, tarefas ou memória (RF-22).
- [ ] CA-09: não há scripts nem suíte em bash no harness, e o `cargo test` passa mais rápido que a suíte bash (RF-23, RNF-02).
- [ ] CA-10: `cargo clippy -- -D warnings` e `cargo fmt --check` passam (RNF-03).

## Referências

| Tipo | Referência | Uso |
|---|---|---|
| arquivo | `packages/harness/scripts/default` | Scripts a reescrever e as mensagens de cada um. |
| arquivo | `packages/harness/scripts/lib` | Lógica compartilhada (front-matter, tarefas, planos, finalização). |
| arquivo | `packages/harness/skills/default/*/scripts` | Scripts das skills e os argumentos atuais. |
| arquivo | `packages/harness/tests/run-tests.sh` | Casos a reproduzir em Rust; tempo atual de cerca de 95 s. |
| arquivo | `packages/harness/config/main.yml` e `packages/harness/config/skills.yml` | Caminhos e pastas de redirecionamento que a inicialização configura. |
| arquivo | `packages/harness/templates/skill/redirect.md` | Formato do redirecionamento. |
| arquivo | `AGENTS.md` e `CLAUDE.md` | Roteamento atual. |
| arquivo | `packages/harness/rules/project/architecture.md` | Regras architecture-07 e 08 sobre a CLI. |
| arquivo | `packages/harness/rules/project/stack.md` | CLI em Rust e scripts em bash até esta spec. |
| memória | DEC-002 | Linguagem Rust, formatos independentes da linguagem e `cargo` na Mac. |
| memória | DEC-001 | Nome `vostok` da CLI. |

## O que mudou

| # | Assunto | Estado no Original | Estado Refinado |
|---|---|---|---|
| 1 | Escopo da migração | Modernizar a CLI, sem dizer quanto. | Reescrita completa dos scripts em subcomandos do `vostok`, com remoção do bash. |
| 2 | Local da instalação | "Deixa de apontar para dentro de packages". | Pasta fixa `vostok/` na raiz do projeto. |
| 3 | Atualização | Não citada. | `vostok update` com separação entre arquivos do binário e do projeto. |
| 4 | Distribuição | Binário para Linux e macOS. | GitHub Releases com quatro binários e `install.sh`. |
| 5 | Testes | Tempo da suíte citado como incômodo. | `cargo test` substitui a suíte bash e precisa ser mais rápido. |
| 6 | Inicialização | Interativa, com configurações opcionais. | Pergunta caminhos de documentos e ferramentas; `--yes` usa os padrões. |
| 7 | Critérios de aceite | 5 critérios gerais. | 11 critérios verificáveis. |
| 8 | Ferramentas | Não citadas. | O usuário escolhe as ferramentas, e só elas recebem roteamento e redirecionamentos. |

## Registro de decisões

| # | Decisão | Alternativa descartada | Motivo |
|---|---|---|---|
| 1 | Migração completa nesta spec. | Binário executando os scripts bash, com reescrita depois. | resposta 1A |
| 2 | Pasta fixa e visível `vostok/`. | `.vostok/` fixa; pasta escolhida na inicialização. | resposta 2C |
| 3 | `vostok update` nesta spec. | Só definir a separação, sem o comando. | resposta 3A |
| 4 | GitHub Releases e `install.sh`. | `cargo install`; pacote npm. | resposta 4A |
| 5 | Testes em Rust e suíte bash aposentada. | Suíte bash contra o binário; as duas. | resposta 5A |
| 6 | Tipo `feature`. | `improvement`. | inferência A |
| 7 | Binário `vostok` num crate em `apps/cli`. | Outro local. | inferência B |
| 8 | Subcomandos espelham os scripts por grupo. | Nomes novos. | inferência C |
| 9 | Formatos de arquivo iguais. | Formatos novos. | inferência D |
| 10 | Inicialização com `--yes`, perguntando caminhos de documentos e ferramentas. | Sem modo não interativo. | inferência E |
| 11 | Inicialização gera o roteamento apontando para `vostok/`. | Roteamento manual. | inferência F |
| 12 | Este projeto move `work/`, `memory/`, `.state/`, regras de projeto e mapa para `vostok/`. | Manter em `packages/harness`. | inferência G |
| 13 | Plataformas Linux e macOS. | Incluir Windows. | inferência H |
| 14 | Cenários de agente rodados de novo nos fluxos que mudaram de comando. | Rodar todos. | inferência I |
| 15 | Manifesto `vostok/.manifest` com versão e hashes, base do update. | Comparar só por data. | inferência J |
| 16 | Arquivo local alterado guardado como `.local` no update. | Recusar o update; sobrescrever. | inferência K |
| 17 | Fixtures geradas pelos scripts atuais como referência de equivalência. | Só testes novos. | inferência L |
| 18 | Só as ferramentas escolhidas recebem roteamento e redirecionamentos; a escolha fica em `skills.yml`. | Instalar para todas. | ajuste 1 do usuário |
| 19 | `--tools` no modo não interativo; sem ele, `--yes` instala para todas as suportadas. | `--yes` sem nenhuma ferramenta. | inferência M |
| 20 | `AGENTS.md` sempre gerado, por concentrar o roteamento lido por Codex e Cursor e apontado pelo `CLAUDE.md`. | Gerar só com Codex ou Cursor. | inferência N |

## Pendências

| # | Pendência | Opções em aberto |
|---|---|---|
| - | Nenhuma | - |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
| 1 | 2026-10-13 | agente | RF-04, RF-05 e RF-06 com a escolha das ferramentas e roteamento só para elas; nova RF-25 (escolha em skills.yml respeitada no update e na criação de skills); novo CB-08; contrato com --tools e a tabela de ferramentas; CT-04 ajustado e novo CT-19; novo CA-11 |
