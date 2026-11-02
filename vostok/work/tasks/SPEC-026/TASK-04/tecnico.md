<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Uma skill padrão, `customize-documents`, que só orquestra os comandos `vostok documents detect` e `vostok documents brand`: o trabalho determinístico fica no `vostok`, como nas outras skills do harness. O ícone é procurado pela skill com Glob, porque a detecção do vostok trata só de cores.

**Decisões.**
- Uma pergunta única com a proposta inteira (cores, nome e ícone); descartado perguntar item a item, que alonga a conversa.
- Nome padrão = nome da pasta do repositório (resposta 3A); descartado o nome do manifesto (`package.json`, `Cargo.toml`), que nem todo projeto tem.
- Contexto da skill só com `config/documents.yml`; as regras de projeto não influenciam a identidade.

**Riscos.** O comportamento da skill depende do agente; a evidência manual mostra os dois caminhos.

**Referências.** RF-01, RF-03, RN-05, RN-06, FA-01 a FA-03, CT-12, CT-13; decisões 2, 3 e 6 da spec.

**Arquivos em comum.** Nenhum: roda sozinha na onda 3.

## O que foi feito

`packages/harness/skills/default/customize-documents/SKILL.md` (1.0.0) orquestra os dois comandos da spec: carrega o contexto, trata o pedido de volta ao padrão (`vostok documents brand --reset`, com confirmação), roda `vostok documents detect`, procura um ícone com Glob (`favicon` e `logo` em SVG, PNG e WebP, fora de `node_modules`, `target`, `dist`, `build` e `vostok`), monta uma única pergunta com a proposta ou pergunta as cores quando não há candidatas e só então roda `vostok documents brand`. As regras `[customize-documents-01]` a `[customize-documents-04]` proíbem gravar sem confirmação, ler arquivos de segredo, customizar as cores de estado e editar `documents.yml` à mão.

A skill entrou no fim da tabela de skills padrão de `packages/harness/context/skill-catalog.md` e no fim das entradas de `packages/harness/config/context.yml`, com `config/documents.yml`. O passo pedia ordem alfabética, mas nenhum dos dois arquivos é ordenado, e as skills anteriores também foram acrescentadas no fim.

Ajustes: o print manual de uma conversa seguindo a skill foi trocado, a pedido do usuário, por duas evidências automáticas em repositórios de exemplo temporários: `skill-comandos.txt` (contexto da skill carregado, `detect` propondo `#0f766e` e `#f59e0b` antes do cinza e `brand` gravando nome, ícone e as duas cores) e `skill-sem-cores.txt` (`detect` sem nenhuma cor, o caso em que a skill pergunta). Ponto de atenção: o comportamento da conversa em si (proposta única, espera da confirmação) só é verificável seguindo a skill com um agente; as evidências cobrem os comandos que ela executa. As suítes `init_update` e `context_memory_rules` seguem passando com a skill nova.
