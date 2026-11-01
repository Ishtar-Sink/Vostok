### Ler

- `vostok/work/specs/refined/SPEC-024-executivo-e-tecnico-preservados-e-escritos-desde-o-planejamento.md` (RF-04, RF-08, RF-09, CT-07, Registro de decisões)
- `packages/harness/skills/default/approve-plan/SKILL.md` (Termos: Descrição; regras approve-plan-07 e approve-plan-09; passo 4)
- `packages/harness/skills/default/finish-task/SKILL.md` (linhas 140 a 160: "Documentos da tarefa")
- `packages/harness/skills/default/approve-task/SKILL.md` (passo 3, linha 79)
- `packages/harness/skills/default/adjust-task/SKILL.md` (linhas 72 e 136)
- `packages/harness/personas/arquiteto.md`
- `packages/harness/templates/task/executivo.md` e `packages/harness/templates/task/tecnico.md`

### Mudar

- `packages/harness/skills/default/approve-plan/SKILL.md` (alterar): escrever o executivo e o técnico no detalhamento
- `packages/harness/skills/default/finish-task/SKILL.md` (alterar): enriquecer os documentos em vez de escrevê-los do zero
- `packages/harness/skills/default/approve-task/SKILL.md` (alterar): a aprovação recusa documento no template
- `packages/harness/skills/default/adjust-task/SKILL.md` (alterar): ajustar os documentos sem nova aprovação, contando como enriquecimento na execução
- `packages/harness/personas/arquiteto.md` (alterar): escreve o `tecnico.md` no detalhamento
- `packages/harness/templates/task/executivo.md` (alterar): comentário do ciclo novo
- `packages/harness/templates/task/tecnico.md` (alterar): comentário do ciclo novo

### Passos

1. `packages/harness/templates/task/executivo.md` e `packages/harness/templates/task/tecnico.md` (alterar): troque o comentário `<!-- Escrito na finalização da tarefa. -->` por `<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->`. Mantenha as variáveis. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test base`.
2. `packages/harness/skills/default/approve-plan/SKILL.md` (alterar): no termo "Descrição", diga que `executivo.md` e `tecnico.md` são escritos no detalhamento, com o que a tarefa vai fazer, e enriquecidos na finalização (skill `finish-task`). No passo 4, troque a frase "`executivo.md` e `tecnico.md` ficam com as variáveis..." por: substitua também `#{TASK_EXECUTIVE}` (até 5 linhas, regra approve-plan-07) e `#{TASK_TECHNICAL}` (abordagem, decisões com alternativas, riscos, referências da spec e arquivos em comum com a onda, regra approve-plan-09); a aprovação recusa a tarefa com eles no template. Na lista "Descrição", tire "só no formato antigo" e cite os dois arquivos ao lado das variáveis do arquivo único.
3. `packages/harness/skills/default/finish-task/SKILL.md` (alterar): em "Documentos da tarefa", troque "são escritos a partir do que foi feito" por "são enriquecidos com o que foi feito": o subagente mantém o que o planejamento escreveu, corrige o que mudou e acrescenta o que foi feito, a partir do `vostok task describe`. No parágrafo do `vostok task finish`, diga que ele recusa documento no template e documento igual ao do início da tarefa (`não foi enriquecido desde o início`).
4. `packages/harness/skills/default/approve-task/SKILL.md` (alterar): no passo 3, diga que a validação recusa a tarefa em pasta com `executivo.md` ou `tecnico.md` no template, e que a correção é escrever o documento (skill `adjust-task`).
5. `packages/harness/skills/default/adjust-task/SKILL.md` (alterar): perto da linha 136, diga que ajustar `executivo.md` ou `tecnico.md` não pede nova aprovação e, numa tarefa em execução, conta como enriquecimento para o `finish`.
6. `packages/harness/personas/arquiteto.md` (alterar): na descrição e nas listas, troque "Detalhes técnicos" por "`tecnico.md` (Detalhes técnicos no arquivo único)", escrito no detalhamento.
7. Rode a Verificação.
