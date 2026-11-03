### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort | grep -v 'atividades'`
- `pnpm --dir apps/painel exec tsc --noEmit && echo tipos-ok`

### Verificação

```bash
# evidência: atividades-testes.txt
cargo test --manifest-path apps/cli/Cargo.toml --features painel --lib painel::read painel::server
cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings
pnpm --dir apps/painel exec tsc --noEmit
```

### Evidências

- `atividades-testes.txt` (automática, da verificação)
  - Prova: a rota reúne as cinco fontes, filtra, ignora avais malformados e responde em até 500 ms com 1.000 atividades (RF-07, CT-09, CT-10, CB-03, CB-04, RNF-01).
  - Trecho: `atividades_reunem_todas_as_fontes`, `atividades_filtram_por_tipo_spec_e_limite`, `avais_malformados_sao_ignorados`, `atividades_respondem_em_ate_500_ms` e `atividades_sao_servidas_sem_token` com `ok`.
  - Falha: algum com `FAILED`, aviso do clippy ou erro do `tsc`.
