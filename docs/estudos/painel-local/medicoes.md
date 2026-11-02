# Medições do painel local

- Data: 2026-11-02
- Versão do `vostok`: 0.2.10
- Commit da worktree do spike: `cb57802` (branch `spike/painel-local`, em `/home/victor/projetos/ai-hub-painel`)
- Versão do `rustc`: 1.98.0
- Node: v24.19.0, pnpm 11.21.0; React 19.3.1 (mesma versão do Atria), Vite 8.3.5, Preact 10.29.8

Este documento registra as medições do spike da TASK-02 da SPEC-027. O código do spike não entra no repositório principal: fica só na worktree.

## Medições do binário e do bundle

Tamanhos em bytes, de `vostok` em `--release --locked`. O tar.gz é gerado como na release (`tar -czf ... -C <dir> vostok`). O bundle é o conteúdo de `apps/painel/dist/`: a soma dos arquivos, o concatenado compactado com gzip -9 e com brotli q11.

| Variante | Binário | tar.gz | Bundle | Bundle gzip | Bundle brotli |
|---|---:|---:|---:|---:|---:|
| base | 4 586 856 | 1 782 243 | - | - | - |
| tiny+react | 5 302 120 | 2 016 614 | 230 201 | 70 936 | 61 219 |
| tiny+preact | 5 102 136 | 1 956 582 | 30 209 | 11 033 | 9 983 |
| axum+react | 5 839 552 | 2 217 820 | 230 201 | 70 936 | 61 219 |
| axum+preact | 5 639 192 | 2 157 510 | 30 209 | 11 033 | 9 983 |

Diferença de cada variante para a `base`:

| Variante | Binário | tar.gz |
|---|---:|---:|
| tiny+react | +715 264 (+15,6%) | +234 371 (+13,2%) |
| tiny+preact | +515 280 (+11,2%) | +174 339 (+9,8%) |
| axum+react | +1 252 696 (+27,3%) | +435 577 (+24,4%) |
| axum+preact | +1 052 336 (+22,9%) | +375 267 (+21,1%) |

Leituras dos números:

- O bundle React pesa 230 KB; o Preact, 30 KB. A troca de React por Preact economiza cerca de 200 KB no binário e no tar.gz, nas duas pilhas HTTP.
- O servidor pesa por si: `tiny_http` adiciona cerca de 485 KB sobre a `base` (515 280 menos o bundle Preact de 30 209). `axum` com `tokio` adiciona cerca de 1,02 MB (1 052 336 menos 30 209). Com o mesmo bundle, `tiny_http` fica quase 570 KB abaixo do `axum`.
- A combinação `tiny+preact` é a menor de todas: 5,10 MB instalado e 1,96 MB compactado.

## Comandos para reproduzir

```bash
# na raiz do repositório principal, com a worktree em /home/victor/projetos/ai-hub-painel
git -C /home/victor/projetos/ai-hub-painel branch --show-current   # spike/painel-local
pnpm --dir /home/victor/projetos/ai-hub-painel/apps/painel install
bash /home/victor/projetos/ai-hub-painel/apps/painel/medir.sh
```

O `medir.sh` gera o bundle de cada sabor (`pnpm build`, e `PAINEL_PREACT=1 pnpm build` para Preact), compila cada variante com `CARGO_TARGET_DIR` próprio e imprime uma linha por variante. A variável `PAINEL_MEDICAO_DIR` escolhe a pasta de trabalho dos builds e dos tar.gz.

Para subir o painel de uma variante: `cargo run --manifest-path /home/victor/projetos/ai-hub-painel/apps/cli/Cargo.toml --features painel-tiny -- painel --sem-navegador`.

## Impacto de Node no build (PQ-03)

- `cargo install --path apps/cli` sem features não muda: a `base` não embute o painel e não precisa de Node.
- Com `painel-tiny` ou `painel-axum`, o `include_dir!` exige `apps/painel/dist/` pronto no momento da compilação. Sem Node e sem `dist/`, o build falha.
- A release (`.github/workflows/release.yml`) precisa de um passo de Node e pnpm antes do `cargo build --release --locked` se o painel entrar no binário.

Duas saídas:

1. Gerar o bundle na CI, com Node e pnpm instalados no workflow de release antes do build do Rust. Mantém o `dist/` fora do repositório.
2. Embutir o `dist/` já gerado, versionado no repositório. Dispensa Node no `cargo install`, ao custo de arquivos gerados no git (cerca de 30 KB com Preact) e de atualizá-los a cada mudança no app.

## Limite proposto

Limite proposto para o painel (PQ-04):

- Binário instalado: no máximo 5,3 MB (5 300 000 bytes). Isso é a `base` mais cerca de 15%, com folga para a variante recomendada, que mede 5,10 MB.
- Compactado (tar.gz da release): no máximo 2,0 MB (2 000 000 bytes). A variante recomendada mede 1,96 MB, com margem de 44 KB.

Justificativa: a `base` mede 4,59 MB e 1,78 MB compactados, e o levantamento registra 4,6 MB e cerca de 1,7 MB. A variante recomendada soma 515 KB ao binário (+11%) e 174 KB ao compactado (+10%). Um limite de 5,3 MB e 2,0 MB aceita essa variante e recusa o `axum` com React (5,84 MB e 2,22 MB), que cresce 27% no binário.

Variante recomendada: `tiny+preact`, com `tiny_http` e o bundle Preact. Ela é a menor medida. O custo é trocar React por Preact no app, o que precisa ser validado em testes de comportamento antes de adotar. Este spike mede tamanho, não comportamento.
