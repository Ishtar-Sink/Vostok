### Baseline

- `cargo test --manifest-path apps/cli/Cargo.toml --test base 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`
- `cargo test --manifest-path apps/cli/Cargo.toml --features painel --test painel_server 2>&1 | grep -E '^test .* (ok|FAILED|ignored)$' | sort`
- `cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings 2>&1 | grep -E '^(warning|error)' | sort`

### Verificação

```bash
# evidência: build-painel-dist.txt
bash apps/cli/scripts/check-painel-dist.sh
cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings
cargo test --manifest-path apps/cli/Cargo.toml --test base
cargo test --manifest-path apps/cli/Cargo.toml --features painel --test painel_server
git status --short apps/painel
```

Resultado esperado: o script imprime `ok` nos cinco cenários e `todos os cenários ok`; o clippy sem avisos; os testes sem `FAILED`; o `git status` sem nada em `apps/painel`.

### Evidências

- `build-painel-dist.txt` (automática, da verificação)
  - Requer: `apps/cli/scripts/check-painel-dist.sh`
  - Espera: todos os cenários ok
  - Tempo-limite: 600
  - Prova: numa cópia isolada do repositório, com diretório alvo temporário e sem rede, o build com a feature `painel` recusa o `dist` mais antigo que `apps/painel/src` quando não há `pnpm`, com a mensagem que cita `pnpm --dir apps/painel build` (RF-08, CT-10, CA-06); aceita o `dist` em dia sem chamar o `pnpm` (CT-11); reconstrói o `dist` velho quando há `pnpm` (inferência H); e, sem a feature ou sem o `dist`, compila como antes (CB-05). O tempo-limite cobre a compilação completa das dependências no diretório temporário (cerca de 1 min) e a reconstrução do painel (cerca de 30 s).
  - Trecho: `ok CT-11 dist em dia`, `ok CT-10 dist velho sem pnpm` seguido da linha do `panicked` com `pnpm --dir apps/painel build`, `ok CB-05 sem a feature`, `ok Reconstrução com pnpm`, `ok CB-05 sem dist` e `todos os cenários ok`.
  - Falha: uma linha `FALHOU <cenário>` com o fim da saída do `cargo build`, por exemplo o build com o `dist` velho terminando sem erro.
