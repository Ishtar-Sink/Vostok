<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** O `Harness` (`apps/cli/src/harness.rs`) ganha `requests_dir`, `questions_dir`, `approvals_dir`, `folders` e `request_dispatch`, que as tarefas seguintes usam em vez de nomes fixos (`"pedidos"`, `"perguntas"`, `"avais.log"`). O `packages/harness/config/main.yml` passa a trazer `paths.requests`, `paths.questions`, `paths.approvals`, a seção `folders` e `request_dispatch: ""`, como no contrato da spec.

**Decisões.**
- Chave ausente vale o padrão em português (nos caminhos, vazio também); um nome de subpasta presente e vazio é erro. Alternativa descartada: exigir as chaves, o que quebraria os projetos da v0.3 até o `vostok update` (decisão 8 da spec, compatibilidade). O mesmo vale para `approval_lock`, lida como desligada quando ausente.
- O leitor de configuração (`support::key_lines`) passa a montar o caminho completo da chave, para ler `folders.requests.pending`. Alternativa descartada: um leitor próprio da seção `folders` em `harness.rs`, que duplicaria o subconjunto de YAML. Os arquivos de configuração lidos por `config_get` (`main.yml`, `tasks.yml`, `specs.yml`, `skills.yml`) têm só dois níveis; o único com três (`models.yml`) é lido pelo próprio `models.rs`, então nada muda para eles. `support.rs` não está nos componentes do plano e entra em "Mudar" por isso.
- Além de nomes repetidos (CB-08), a validação recusa nomes vazios, com `/` ou `..`: o nome é um segmento de pasta e não pode sair da pasta configurada.
- O estado gravado nos arquivos continua com os valores atuais (`pendente`, `aberta`...); só o nome da pasta é configurável (RN-06, decisão 11).
- `vostok/config/main.yml` deste repositório não entra em "Mudar": ele é o harness instalado e recebe as chaves novas pelo `vostok update` (`merge_keys` acrescenta os filhos de `paths` e os blocos novos), num commit próprio de atualização, como aconteceu com `approval_lock`.

**Riscos.** Uma configuração de projeto com três níveis lida antes como `pai.neto` mudaria de caminho; nenhum arquivo distribuído tem isso. `request_dispatch` só é lido e validado aqui; o disparo é da TASK-07.

**Referências da spec.** RF-16, RF-13 (chave), RN-06, CB-07, CB-08, CT-19, CT-20, decisões 8, 10 e 11.

**Arquivos em comum com a onda.** Nenhum: na onda 1, a TASK-12 só mexe no frontend do quadro.
