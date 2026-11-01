### Ler

- `vostok/work/specs/refined/SPEC-025-documentos-finais-com-visual-e-conteudo-profissionais.md` (RF-05 a RF-09, RN-01, RN-02, RNF-05, Registro de decisões 2 e 6)
- `docs/estudos/documentos-finais.md` (da TASK-01)
- `packages/harness/templates/finalize/content.md` e `packages/harness/skills/default/finalize-spec/SKILL.md` (modelo de conteúdo atual)
- `docs/specs/SPEC-023/executivo.html`, `docs/specs/SPEC-023/tecnico.html` e `vostok/work/.archive/SPEC-023/content.md` (conteúdo atual de uma spec real)

### Mudar

- `docs/estudos/documentos-finais.md` (alterar): seção `## Estudo de conteúdo`

### Passos

1. Pesquise referências públicas sobre leitura na tela e escrita de documentos para executivos e para manutenção: varredura de texto, prosa contra listas, tabelas, tamanho de parágrafo, pirâmide invertida, documentação de decisões (ADR) e de KT (runbook). Cite cada referência com o link e o que ela sustenta.
2. Analise o conteúdo atual da SPEC-023: o que falta para um leitor não técnico e para quem vai dar sustentação, com exemplos.
3. `docs/estudos/documentos-finais.md` (alterar): na seção `## Estudo de conteúdo`, escreva em prosa: as referências; uma regra de formato (quando prosa, quando lista, quando tabela, e o limite de listas), que considere a preferência do usuário por menos bullets; as seções do executivo (objetivo, o que foi entregue, valor e impacto, ganhos secundários, próximos passos quando se aplicam), do técnico (o que foi feito, por que, como, operação e diagnóstico, mapa do código, limites e dívidas, e as que o estudo achar necessárias) e das evidências; e, para cada seção, o propósito, o formato e o tamanho esperado (RNF-05).
4. Diga quais regras o `vostok finalize validate` pode conferir (seção presente, tamanho, termos proibidos no executivo) e quais ficam como orientação na skill. Confira: `grep -n '^### ' docs/estudos/documentos-finais.md`.
