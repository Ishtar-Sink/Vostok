### Baseline

- `ls docs/estudos | sort`

### Verificação

```bash
# esperado: o estudo existe e tem as seções dos passos 1 a 8
test -f docs/estudos/painel-ponta-a-ponta.md && echo estudo-ok
```

### Evidências

- `estudo.diff` (diff): `docs/estudos/painel-ponta-a-ponta.md`
  - Prova: o estudo cobre os sete temas do RF-01, com medições, e a proposta da SPEC-033 (RF-01, CT-01, RNF-04).
  - Trecho: as seções "Técnicas para operar sem o chat", "Processos a operar pelo painel", "Status das entidades", "Perguntas do agente", "Arquivos de comunicação", "Fontes de atividade" e "Proposta da SPEC-033", com tabelas de medição.
  - Falha: seção ausente, comparação sem números ou proposta sem escopo.
