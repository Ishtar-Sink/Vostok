---
id: TASK-04
spec: SPEC-019
title: "Medição do consumo antes e depois"
profile: padrao
persona: qa
depends_on: [TASK-01, TASK-02, TASK-03]
wave: 3
covers: [CT-10]
type: tarefa
labels: [SPEC-019, padrao]
status: done
content_hash: 8d523bd8a5631f50c7660221856921641c637a17353e4773be732044cda3e1ff
adjustments: 0
adjusted_at: null
approved_at: 2026-10-31T10:32:31Z
approved_hash: 365d0098f2cfe5af3938eeb5a63e4ca0f767a29ac241ae79ec39030ad9256219
attempts: 0
tests_result: passed
regression_result: passed
evidence_result: passed
model: sonnet
---

# TASK-04: Medição do consumo antes e depois

## Execução

### Ler

- `vostok/work/specs/refined/SPEC-019-reduzir-o-consumo-de-tokens-dos-subagentes.md` (RNF-01, CT-10)
- `docs/estudos/consumo-de-tokens.md` (seções "Como medir" e "Totais observados")

### Mudar

- `docs/estudos/consumo-de-tokens.md` (alterar): seção "Medição antes e depois" com os números do `/usage`.

### Passos

1. Instale a versão com a SPEC-019 (`cargo install --path apps/cli` e `vostok update`).
2. Prepare duas cópias descartáveis do repositório fora do projeto (`git worktree add <pasta> v0.2.3` e `git worktree add <pasta> HEAD`), cada uma com o `vostok` da versão correspondente (`cargo install --path apps/cli --root <pasta>/.bin` e o `PATH` da sessão apontando para ele).
3. Em cada cópia, crie a mesma tarefa média da CLI: uma spec `improvement` com uma tarefa que acrescenta a `vostok task status` uma linha `Total de tarefas: N`, com teste em `apps/cli/tests/task.rs` (mesmo texto de spec e de tarefa nas duas cópias, aprovada até `approved`).
4. Em cada cópia, numa sessão nova do Claude Code (`/clear` antes), peça "executa a TASK-01 com subagente" na 0.2.3 e "executa a TASK-01" no fluxo novo, e ao terminar anote do `/usage`: o bloco da sessão, a atribuição a subagentes e a linha do cache, e o total da notificação do subagente, se houver.
5. Grave os números na evidência manual `medicao-usage.md` da tarefa e acrescente ao estudo a seção "Medição antes e depois" com a tabela (modo, sessão, subagentes, total) e a conclusão.
6. Remova as cópias (`git worktree remove`).

### Baseline

- `cargo run --quiet --manifest-path apps/cli/Cargo.toml -- rules validate`

### Verificação

```bash
f=vostok/work/tasks/SPEC-019/evidence/TASK-04/medicao-usage.md
test -s "$f"
grep -q '0.2.3' "$f"
grep -qi 'fluxo novo' "$f"
grep -q '^## Medição antes e depois' docs/estudos/consumo-de-tokens.md
```

Resultado esperado: a evidência existe com os dois modos e o estudo tem a seção nova.

### Evidências

- `medicao-usage.md` (manual): a saída do `/usage` (sessão, atribuição a subagentes e cache) dos dois modos, com a data, a versão e o modelo de cada sessão.
  - Prova: o consumo do fluxo novo é menor que o da 0.2.3 para a mesma tarefa (RNF-01, CT-10).
  - Trecho: os totais de cada modo e a diferença em porcentagem.
  - Falha: um dos modos sem número, ou o fluxo novo com consumo maior ou igual.

### Concluída quando

- [x] A mesma tarefa média da CLI gasta menos no fluxo novo que na 0.2.3, com os números do `/usage` registrados (RNF-01, CT-10).

  > Como comprovar: abrir `medicao-usage.md` e a seção "Medição antes e depois" do estudo e comparar os totais dos dois modos.

## Descrição

### Resumo executivo

Mede, com o painel de uso do Claude Code, quanto a mesma tarefa gasta antes e depois das mudanças, para confirmar a economia com números. Perfil padrão; depende das três tarefas anteriores e de sessões novas do Claude Code.

### Detalhes técnicos

- **Abordagem:** duas cópias descartáveis (0.2.3 e fluxo novo), a mesma tarefa média, sessões novas e `/usage` ao fim de cada uma.
- **Decisões:** usuário 4A (medição manual somada aos critérios determinísticos); a medição fica no estudo.
- **Riscos:** variação entre execuções; registrar modelo e versão de cada sessão.
- **Referências:** RNF-01, CT-10.
- **Arquivos em comum:** nenhum.

### Card

- **Título:** Medição do consumo antes e depois
- **Tipo:** tarefa
- **Descrição:** Mede o consumo da mesma tarefa na 0.2.3 e no fluxo novo e registra os números no estudo.
- **Critérios de aceite:**
  - [ ] Medição dos dois modos com /usage
  - [ ] Números registrados no estudo
- **Rótulos:** SPEC-019, padrao
- **Estimativa:** padrao
- **Bloqueada por:** TASK-01, TASK-02, TASK-03

## Registro de execução

| # | Data | Etapa | Resultado | Detalhe |
|---|---|---|---|---|
| 1 | 2026-10-31 | início | in_progress | aprovada em 2026-10-31T10:32:31Z; modelo: sonnet |
| 2 | 2026-10-31 | baseline | coletado | 1 comando(s); códigos de saída: 0 |
| 3 | 2026-10-31 | testes | passed | Verificação sem falhas; saída em validation/tests.out |
| 4 | 2026-10-31 | regressão | passed | 1 comando(s) iguais ao baseline |
| 5 | 2026-10-31 | evidências | passed | 1 evidência(s) em evidence/TASK-04/ |
| 6 | 2026-10-31 | conclusão | done | testes, regressão e evidências passed; critérios de conclusão marcados |

## Histórico de ajustes

| # | Data | Origem | Resumo |
|---|---|---|---|
