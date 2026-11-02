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

## O que foi feito

`apps/cli/src/commands/finalize.rs` lê o grupo `brand` de `documents.yml` (ausente ou vazio vale a identidade atual) e acrescenta aos valores do cabeçalho `BRAND_NAME`, o nome escapado para HTML ou `Vostok`, e `BRAND_MARK`, o `<i></i>` do gradiente ou um `<img class="brand-mark" alt="">` com o ícone em `data:` (SVG, PNG ou WebP pela extensão, com a função `base64` que já embute as fontes). O caminho do ícone é relativo à raiz do projeto; ausente, acima de 256 KB ou de outra extensão é erro do build com o caminho e o motivo. Os três templates trocaram a marca fixa pelas duas variáveis, com `.brand img` dimensionado ao lado de `.brand i`, e o de evidências ganhou `white-space: nowrap` na primeira coluna das tabelas de `#p-cobertura`. Nenhuma cor literal foi acrescentada.

Os testes `brand_*` (nome escapado, ícone embutido, ícone ausente, grande ou de extensão não suportada, identidade vazia) e `coverage_ids_do_not_wrap` em `apps/cli/tests/finalize.rs` usam os auxiliares `filled_folder_project` e `project_with_brand`, que substitui o grupo `brand` já presente no `documents.yml` da TASK-01. Os goldens passam sem fixture alterada, porque os templates instalados sem as variáveis seguem como antes.

Ajuste: o print `executivo-marca.png` acrescentava um segundo grupo `brand` ao `documents.yml` e o grupo vazio da TASK-01 prevalecia; o comando passou a trocar os valores do grupo existente. Os dois prints (nome e ícone no topo, IDs da cobertura numa linha) foram conferidos pelo usuário. Ponto de atenção: o tema escuro não foi conferido em print.
