<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Traz do spike a tela de configurações (`schema.ts`, `yaml.ts`, `fields.tsx` e a página) e liga o salvar à rota `PUT /api/configuracao` da TASK-06. Cobre RF-03, RF-10 e CT-11.

**Decisões.** O YAML é reescrito pelo `yaml.ts`, que troca só os valores mudados e preserva comentários e ordem. Gerar o YAML do zero foi descartado, porque perde os comentários que explicam cada opção. O conflito reaproveita o diálogo de `shared/`.

**Riscos.** Um formato de YAML que o `yaml.ts` não entende: a chave vai para "Avançado" como texto, sem perder o valor.

**Arquivos em comum na onda.** Nenhum: cada tarefa de telas muda só as próprias pastas de feature, e as ações compartilhadas já estão em `shared/` (TASK-03).
