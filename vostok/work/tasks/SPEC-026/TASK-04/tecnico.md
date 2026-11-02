<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** Uma skill padrão, `customize-documents`, que só orquestra os comandos `vostok documents detect` e `vostok documents brand`: o trabalho determinístico fica no `vostok`, como nas outras skills do harness. O ícone é procurado pela skill com Glob, porque a detecção do vostok trata só de cores.

**Decisões.**
- Uma pergunta única com a proposta inteira (cores, nome e ícone); descartado perguntar item a item, que alonga a conversa.
- Nome padrão = nome da pasta do repositório (resposta 3A); descartado o nome do manifesto (`package.json`, `Cargo.toml`), que nem todo projeto tem.
- Contexto da skill só com `config/documents.yml`; as regras de projeto não influenciam a identidade.

**Riscos.** O comportamento da skill depende do agente; a evidência manual mostra os dois caminhos.

**Referências.** RF-01, RF-03, RN-05, RN-06, FA-01 a FA-03, CT-12, CT-13; decisões 2, 3 e 6 da spec.

**Arquivos em comum.** Nenhum: roda sozinha na onda 3.
