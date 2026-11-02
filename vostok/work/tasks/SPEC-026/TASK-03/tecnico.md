<!-- Escrito no planejamento com o que a tarefa vai fazer e enriquecido na conclusão com o que foi feito. -->

**Abordagem.** O build lê o grupo `brand` do `documents.yml` e passa duas variáveis novas aos templates, `BRAND_NAME` e `BRAND_MARK`. O ícone é embutido em base64 com a função que já embute as fontes. Os templates trocam a marca fixa pelas variáveis e ganham a regra `nowrap` da primeira coluna da cobertura.

**Decisões.**
- Ícone embutido como `<img>` com `data:` (inferência H); descartado colar o SVG inline, que exporia scripts e estilos do arquivo ao documento.
- Ícone inválido é erro do build, com o caminho (CB-02); descartado cair no quadrado silenciosamente, que esconderia a configuração quebrada.
- Grupo ausente ou vazio vale a identidade atual (RF-08), o que mantém os goldens, que reconstroem um harness antigo (RNF-04).
- Persona ui-ux-designer: a tarefa muda o cabeçalho visual, e os prints são conferidos pelo usuário na finalização.

**Riscos.** Ícones com proporção muito diferente de 1:1 ficam pequenos; `object-fit: contain` evita distorção.

**Referências.** RF-06 a RF-08, RF-12, RNF-01, RNF-02, RNF-04, RNF-05, RN-04, CT-05 a CT-07, CT-11, CB-02, CB-03, CB-08; decisões 3, 10 e 12 da spec.

**Arquivos em comum com a onda 1.** Nenhum: a TASK-01 não toca `finalize.rs`, os templates nem `tests/finalize.rs`. Os testes desta tarefa escrevem o grupo `brand` direto no YAML do projeto de teste.
