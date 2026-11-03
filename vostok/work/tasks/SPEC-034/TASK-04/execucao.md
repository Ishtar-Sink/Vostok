### Ler

- `vostok/work/specs/refined/SPEC-034-corrigir-o-fluxo-do-painel-antes-da-v0-4-0.md` (RF-08, CB-05, CT-10, CT-11, CA-06, o risco do ciclo de desenvolvimento e a decisão 13, inferência H)
- `apps/cli/build.rs` (fn prepare_painel_bundle, fn build_painel)
- `apps/cli/src/painel/server.rs` (const HAS_BUNDLE, const BUNDLE_MISSING e o `include_dir!` de `../painel/dist`: o que o cfg `painel_bundle` liga)
- `apps/painel/package.json` (script `build`: `tsc --noEmit && vite build`)
- `apps/painel/scripts/print.sh` (exemplo de script de verificação do repositório em bash)
- `.github/workflows/release.yml` (passos `pnpm --dir apps/painel install --frozen-lockfile`, `pnpm --dir apps/painel build` e `cargo build --release --locked --features painel`)

### Mudar

- `apps/cli/build.rs` (alterar): com a feature `painel`, reconstruir o `dist` mais antigo que `apps/painel/src`, `index.html` ou `package.json` e recusar o build quando a reconstrução falhar (RF-08); sem a feature ou sem o `dist`, nada muda (CB-05)
- `apps/cli/scripts/check-painel-dist.sh` (criar): confere o `build.rs` numa cópia isolada do repositório, com `CARGO_TARGET_DIR` temporário e sem rede, nos cenários CT-10, CT-11, CB-05 e da reconstrução automática

### Passos

1. `apps/cli/scripts/check-painel-dist.sh` (criar, com `chmod +x`): script bash com `set -euo pipefail` e um comentário de cabeçalho que cita a SPEC-034 e os cenários. Ele não toca o repositório:
   - `root` é a raiz do repositório (`"$(cd "$(dirname "$0")/../../.." && pwd)"`); `work="$(mktemp -d)"` com `trap 'rm -rf "$work"' EXIT`; `export CARGO_TARGET_DIR="$work/target"`.
   - Copia com `rsync -a` para `$work`: `apps/cli` sem `target`, `apps/painel` sem `node_modules` e sem `dist`, e `packages/harness`, nos mesmos caminhos relativos (o `build.rs` usa `../painel` e `../../packages/harness`).
   - Cria em `$work/apps/painel/dist/index.html` um HTML mínimo (`<!doctype html><title>painel</title>`), para não depender do `dist` do repositório.
   - `nopnpm="$(dirname "$(command -v cargo)"):$(dirname "$(command -v rustc)"):/usr/bin:/bin"`; se `env PATH="$nopnpm" sh -c 'command -v pnpm'` achar o `pnpm`, termina com `pnpm continua no PATH sem pnpm` e código 1.
   - `old` marca os arquivos e pastas dados com `touch -d '2001-01-01 00:00:00'` (pelo `find ... -exec touch ... {} +`); `build <PATH> [args]` roda `env PATH=<PATH> cargo build --offline --manifest-path "$work/apps/cli/Cargo.toml" [args]`, com a saída em `$work/out`, e devolve o código; cada cenário o chama como `code=0; build ... || code=$?`, para o `set -e` não interromper o script.
   - Cenários, nesta ordem, cada um imprime `ok <nome>: <resultado>` ou `FALHOU <nome>: <resultado>` seguido de `tail -n 30 "$work/out"` e termina com código 1:
     1. `CT-11 dist em dia`: `old` em `src`, `index.html` e `package.json` da cópia, `touch` no `dist/index.html`; `build "$nopnpm" --features painel` termina com 0.
     2. `CT-10 dist velho sem pnpm`: `old` no `dist/index.html`, `touch` em `src/main.tsx`; `build "$nopnpm" --features painel` termina com código diferente de 0 e `$work/out` contém `pnpm --dir apps/painel build`; imprime também a linha da saída com esse texto (`grep -F -m1`).
     3. `CB-05 sem a feature`: com o `dist` ainda velho, `build "$nopnpm"` (sem `--features`) termina com 0.
     4. `Reconstrução com pnpm`: `cp -a "$root/apps/painel/node_modules" "$work/apps/painel/"` (sem ela, `FALHOU` citando `rode pnpm --dir apps/painel install`), `export npm_config_offline=true`; com o `dist` velho, `build "$PATH" --features painel` termina com 0 e `$work/apps/painel/dist/index.html` fica mais novo (`-nt`) que `src/main.tsx`.
     5. `CB-05 sem dist`: `rm -rf "$work/apps/painel/dist" "$work/apps/painel/node_modules"`; `build "$nopnpm" --features painel` termina com 0 e `$work/out` contém `painel sem bundle`.
   - No fim, imprime `todos os cenários ok`.
   Confira: `bash apps/cli/scripts/check-painel-dist.sh` termina com `FALHOU CT-10 dist velho sem pnpm` antes da mudança do passo 2 (o `build.rs` atual aceita o `dist` velho), depois de `ok CT-11 dist em dia`.
2. `apps/cli/build.rs` (alterar):
   - Acrescente `const PAINEL_WATCHED: &[&str] = &["src", "index.html", "package.json"];` e use-a no laço dos `cargo:rerun-if-changed` de `fn prepare_painel_bundle`. Não acrescente o `dist` aos vigiados.
   - Crie `fn newest_modification(path: &Path, newest: &mut Option<SystemTime>)`: lê `fs::metadata(path)`; se falhar, ignora; guarda o `modified()` mais novo; se for pasta, chama a si mesma em cada entrada do `fs::read_dir` (a data da própria pasta também conta: apagar um arquivo de `src` deixa o `dist` velho).
   - Crie `fn bundle_is_outdated(painel: &Path, index: &Path) -> bool`: verdadeiro quando o `modified()` de `index` é estritamente menor que o mais novo de `PAINEL_WATCHED` (por `newest_modification`); falso quando alguma das datas não pode ser lida.
   - Em `fn prepare_painel_bundle`, mantenha o ramo de `!index.is_file()` como está (aviso `painel sem bundle` e segue, CB-05) e acrescente o ramo `else if bundle_is_outdated(painel, &index)`: chama `build_painel(painel)`; no `Err(reason)`, ou se depois dele `index` ainda não for arquivo, `panic!("o bundle do painel em {} é mais antigo que o código de apps/painel e não foi reconstruído ({reason}); rode `pnpm --dir apps/painel build` e compile de novo", painel.join("dist").display())` (no caso sem `index`, `reason` é `a reconstrução não gerou dist/index.html`).
   - Atualize o comentário de `fn prepare_painel_bundle` e o do topo do arquivo: sem Node e sem `dist`, compila sem o bundle (RF-12 da SPEC-029); com `dist` velho, reconstrói ou recusa (RF-08 da SPEC-034).
   - Importe `std::time::SystemTime`. `build_painel` não muda.
   Confira: `bash apps/cli/scripts/check-painel-dist.sh` termina com `todos os cenários ok`.
3. `apps/cli` (conferir): `cargo clippy --manifest-path apps/cli/Cargo.toml --features painel --all-targets -- -D warnings` sem avisos (cobre o `build.rs`), `cargo test --manifest-path apps/cli/Cargo.toml --test base` e `cargo test --manifest-path apps/cli/Cargo.toml --features painel --test painel_server` sem falhas, e `git status --short apps/painel` sem nada novo (o script não muda o repositório).
