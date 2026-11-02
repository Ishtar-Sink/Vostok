<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Traz do spike a tela de configurações (`schema.ts`, `yaml.ts`, `fields.tsx` e a página) e liga o salvar à rota `PUT /api/configuracao` da TASK-06. Cobre RF-03, RF-10 e CT-11.

**Decisões.** O YAML é reescrito pelo `yaml.ts`, que troca só os valores mudados e preserva comentários e ordem. Gerar o YAML do zero foi descartado, porque perde os comentários que explicam cada opção. O conflito reaproveita o diálogo de `shared/`.

**Riscos.** Um formato de YAML que o `yaml.ts` não entende: a chave vai para "Avançado" como texto, sem perder o valor.

**Arquivos em comum na onda.** Nenhum: cada tarefa de telas muda só as próprias pastas de feature, e as ações compartilhadas já estão em `shared/` (TASK-03).


## O que foi feito

**O que mudou:**
- `features/settings` traz do spike `settings-page.tsx`, `schema.ts`, `yaml.ts`, `fields.tsx` e os CSS.
- **`writeConfig`:** reescreve só as chaves editadas, preservando comentários, listas e blocos `|`.
- **Salvar:** chama `PUT /api/configuracao` e mostra os problemas na barra; um 409 abre o diálogo com "Seu texto" e "Versão no disco".
- **Testes:** 16, que falharam antes do código.

**Pontos de atenção:**
- `/api/configuracoes` não devolve o `hash`. A tela calcula o SHA-256 no navegador com `crypto.subtle`, que funciona em localhost.
- O diálogo de 409 foi feito na própria feature, porque o `shared/edit-dialog` só edita `.md`.
- Num conflito, só há "Usar a versão do disco". A opção "gravar por cima", da RN-06, falta nesta tela.

**Verificação:**
- `tsc`, vitest e build passaram.
- A regressão não encontrou diferença.
- O teste gravou `task-10-testes.txt`, e o print `configuracoes.png` foi conferido pelo usuário.
