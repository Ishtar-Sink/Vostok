### Ler

- `vostok/work/specs/refined/SPEC-026-documentos-finais-com-a-identidade-visual-do-repositorio.md` (RF-01, RF-03, RN-06, "Fluxos", CT-12, CT-13)
- `packages/harness/skills/default/finalize-spec/SKILL.md` (formato de skill padrão: front-matter, Persona, Termos, Regras, Checklist, Passos)
- `packages/harness/templates/skill/skill.md` (template de skill)
- `packages/harness/context/skill-catalog.md` (linhas das skills padrão)
- `packages/harness/config/context.yml` (entradas por skill)
- `apps/cli/src/commands/documents.rs` (saídas de `detect` e `brand`, das TASK-01 e TASK-02)

### Mudar

- `packages/harness/skills/default/customize-documents/SKILL.md` (criar): a skill
- `packages/harness/context/skill-catalog.md` (alterar): linha da skill nova
- `packages/harness/config/context.yml` (alterar): `customize-documents` com `config/documents.yml`

### Passos

1. `packages/harness/skills/default/customize-documents/SKILL.md` (criar), a partir de `templates/skill/skill.md`, versão `1.0.0`, `allowed-tools: Read Glob Bash(vostok *)`, com a descrição: "Customiza a identidade visual dos documentos finais do repositório: detecta as cores do projeto com vostok documents detect, propõe a cor principal, a secundária, o nome e o ícone, pergunta quando não encontra nada e, só com a confirmação do usuário, grava com vostok documents brand. Use quando o usuário pedir para customizar, personalizar ou aplicar a identidade, a marca, as cores ou o logo nos documentos finais, ou para voltar ao visual padrão." Conteúdo:
   - Regras: `[customize-documents-01]` nunca grava sem a confirmação do usuário (RN-06); `[customize-documents-02]` nunca lê arquivos de segredo para achar cores (RN-05); `[customize-documents-03]` as cores de estado não são customizáveis (RN-01).
   - Passos: (1) `vostok context load --skill customize-documents`; (2) pedido de volta ao padrão: confirme com o template `question` e rode `vostok documents brand --reset` (FA-03); (3) rode `vostok documents detect`; (4) com candidatas, proponha a principal e a secundária mais relevantes, o nome (padrão: nome da pasta do repositório) e um ícone (procure com Glob `**/favicon.svg`, `**/favicon.png`, `**/logo.svg`, `**/logo.png`, fora de `node_modules`, `target` e `vostok`), numa única pergunta com o template `question`; sem candidatas, pergunte a principal (obrigatória), a secundária, o nome e o ícone (opcionais) (RF-03, FA-01); **PARE** e aguarde; (5) recusa: pare sem gravar (FA-02); (6) rode `vostok documents brand --primary ... [--secondary ...] [--name ...] [--icon ...]`; erro: responda com o template `blocker` com a mensagem; (7) responda com o template `completion`, com os tokens alterados, os avisos de contraste e a oferta de gerar uma prévia numa spec arquivada.
2. `packages/harness/context/skill-catalog.md` (alterar): acrescente, em ordem alfabética entre as skills padrão, `| customize-documents | <descrição> | \`../skills/default/customize-documents\` |`, no formato das outras linhas.
3. `packages/harness/config/context.yml` (alterar): acrescente `customize-documents:` com `- config/documents.yml`, em ordem alfabética.
4. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test init_update` (catálogo, redirecionamentos e contexto das skills padrão) e `cargo test --manifest-path apps/cli/Cargo.toml --test context_memory_rules`.
