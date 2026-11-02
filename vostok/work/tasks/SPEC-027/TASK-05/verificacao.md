### Baseline

- `ls docs/estudos | sort`

### Verificação

```bash
grep -c '^### PQ-' docs/estudos/painel-local.md
grep -n '^## ' docs/estudos/painel-local.md
git status --short -- apps packages
```

Resultado esperado: `12`; as seções do passo 2; nenhuma linha em `apps` ou `packages`.

### Evidências

- `estudo.diff` (diff): `docs/estudos/painel-local.md`
  - Prova: a resposta a cada pergunta, a conclusão de viabilidade, as medições, a arquitetura recomendada e as specs propostas (PQ-01 a PQ-12, CA-01, CA-02, CA-03, CA-04, CA-11, CA-12).
  - Trecho: as 12 subseções `### PQ-`, a seção `## Conclusão` e a lista de `## Specs propostas`.
  - Falha: pergunta sem resposta ou conclusão sem motivo.
  - Espera: `## Conclusão`
- `sem-spike.txt` (automática): `git status --short -- apps packages; git ls-files -- 'apps/painel' 'apps/cli/src/commands/painel.rs' | wc -l`
  - Prova: nenhum arquivo do protótipo entrou no repositório principal (CA-13).
  - Trecho: nenhuma linha de `git status` e a contagem `0`.
  - Falha: `apps/painel` ou `painel.rs` listados.
