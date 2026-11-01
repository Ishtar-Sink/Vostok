# Fontes dos documentos finais

Subconjuntos da Inter e da Outfit, licença OFL (`OFL-Inter.txt` e `OFL-Outfit.txt`), embutidos em base64 pelo `vostok finalize build`, sem requisição externa. Origem: o pacote de tema do Atria (`Ishtar-Sink/atria`, `packages/theme/fonts/`), que segue `Ishtar-Sink/.github`, `docs/identidade-ecossistema.md`.

| Arquivo | Origem | Tamanho |
|---|---|---|
| `Inter.woff2` | `InterVariable.woff2` (352 KB) | 99,5 KB |
| `Outfit.woff2` | `Outfit-Variable.ttf` (111 KB) | 33,4 KB |

As duas continuam variáveis (pesos de 100 a 900). Comando usado em cada uma:

```bash
uvx --from 'fonttools[woff]' pyftsubset <origem> \
  --unicodes='U+0000-00FF,U+0131,U+0152-0153,U+02C6,U+02DA,U+02DC,U+2000-206F,U+20AC,U+2122,U+2190-2193,U+2212' \
  --layout-features='*' --flavor=woff2 --output-file=<destino>
```
