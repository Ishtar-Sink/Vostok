### Baseline

- `ls docs/estudos | sort` (muda): a pasta dos protótipos entra na lista

### Verificação

```bash
ls docs/estudos/documentos-finais/modelo-a docs/estudos/documentos-finais/modelo-b docs/estudos/documentos-finais/modelo-c
! grep -l 'https://' docs/estudos/documentos-finais/modelo-*/*.html
```

Resultado esperado: seis páginas, e nenhuma com URL externa.

### Evidências

- `prototipo-a-exec.png` (print): `CHROME="$(ls -d ~/.cache/ms-playwright/chromium-*/chrome-linux64/chrome | sort | tail -1)"; "$CHROME" --headless=new --no-sandbox --disable-gpu --hide-scrollbars --window-size=1280,3600 --screenshot="$VOSTOK_EVIDENCE/prototipo-a-exec.png" "file://$PWD/docs/estudos/documentos-finais/modelo-a/executivo.html"`
  - Prova: o modelo A no executivo (RF-02, RF-03, CT-02).
  - Trecho: o documento inteiro, com o conteúdo da SPEC-023 reescrito.
  - Falha: página em branco ou sem o layout do modelo.
  - Requer: `docs/estudos/documentos-finais`
- `prototipo-a-tec.png` (print): `CHROME="$(ls -d ~/.cache/ms-playwright/chromium-*/chrome-linux64/chrome | sort | tail -1)"; "$CHROME" --headless=new --no-sandbox --disable-gpu --hide-scrollbars --window-size=1280,3600 --screenshot="$VOSTOK_EVIDENCE/prototipo-a-tec.png" "file://$PWD/docs/estudos/documentos-finais/modelo-a/tecnico.html"`
  - Prova: o modelo A no tecnico (RF-02, RF-03, CT-02).
  - Trecho: o documento inteiro, com o conteúdo da SPEC-023 reescrito.
  - Falha: página em branco ou sem o layout do modelo.
  - Requer: `docs/estudos/documentos-finais`
- `prototipo-a-celular.png` (print): `CHROME="$(ls -d ~/.cache/ms-playwright/chromium-*/chrome-linux64/chrome | sort | tail -1)"; "$CHROME" --headless=new --no-sandbox --disable-gpu --hide-scrollbars --window-size=400,2400 --screenshot="$VOSTOK_EVIDENCE/prototipo-a-celular.png" "file://$PWD/docs/estudos/documentos-finais/modelo-a/executivo.html"`
  - Prova: o modelo A no celular, sem rolagem horizontal (RF-02, CT-02).
  - Trecho: o executivo em 400 px.
  - Falha: conteúdo cortado ou mais largo que a tela.
  - Requer: `docs/estudos/documentos-finais`
- `prototipo-b-exec.png` (print): `CHROME="$(ls -d ~/.cache/ms-playwright/chromium-*/chrome-linux64/chrome | sort | tail -1)"; "$CHROME" --headless=new --no-sandbox --disable-gpu --hide-scrollbars --window-size=1280,3600 --screenshot="$VOSTOK_EVIDENCE/prototipo-b-exec.png" "file://$PWD/docs/estudos/documentos-finais/modelo-b/executivo.html"`
  - Prova: o modelo B no executivo (RF-02, RF-03, CT-02).
  - Trecho: o documento inteiro, com o conteúdo da SPEC-023 reescrito.
  - Falha: página em branco ou sem o layout do modelo.
  - Requer: `docs/estudos/documentos-finais`
- `prototipo-b-tec.png` (print): `CHROME="$(ls -d ~/.cache/ms-playwright/chromium-*/chrome-linux64/chrome | sort | tail -1)"; "$CHROME" --headless=new --no-sandbox --disable-gpu --hide-scrollbars --window-size=1280,3600 --screenshot="$VOSTOK_EVIDENCE/prototipo-b-tec.png" "file://$PWD/docs/estudos/documentos-finais/modelo-b/tecnico.html"`
  - Prova: o modelo B no tecnico (RF-02, RF-03, CT-02).
  - Trecho: o documento inteiro, com o conteúdo da SPEC-023 reescrito.
  - Falha: página em branco ou sem o layout do modelo.
  - Requer: `docs/estudos/documentos-finais`
- `prototipo-b-celular.png` (print): `CHROME="$(ls -d ~/.cache/ms-playwright/chromium-*/chrome-linux64/chrome | sort | tail -1)"; "$CHROME" --headless=new --no-sandbox --disable-gpu --hide-scrollbars --window-size=400,2400 --screenshot="$VOSTOK_EVIDENCE/prototipo-b-celular.png" "file://$PWD/docs/estudos/documentos-finais/modelo-b/executivo.html"`
  - Prova: o modelo B no celular, sem rolagem horizontal (RF-02, CT-02).
  - Trecho: o executivo em 400 px.
  - Falha: conteúdo cortado ou mais largo que a tela.
  - Requer: `docs/estudos/documentos-finais`
- `prototipo-c-exec.png` (print): `CHROME="$(ls -d ~/.cache/ms-playwright/chromium-*/chrome-linux64/chrome | sort | tail -1)"; "$CHROME" --headless=new --no-sandbox --disable-gpu --hide-scrollbars --window-size=1280,3600 --screenshot="$VOSTOK_EVIDENCE/prototipo-c-exec.png" "file://$PWD/docs/estudos/documentos-finais/modelo-c/executivo.html"`
  - Prova: o modelo C no executivo (RF-02, RF-03, CT-02).
  - Trecho: o documento inteiro, com o conteúdo da SPEC-023 reescrito.
  - Falha: página em branco ou sem o layout do modelo.
  - Requer: `docs/estudos/documentos-finais`
- `prototipo-c-tec.png` (print): `CHROME="$(ls -d ~/.cache/ms-playwright/chromium-*/chrome-linux64/chrome | sort | tail -1)"; "$CHROME" --headless=new --no-sandbox --disable-gpu --hide-scrollbars --window-size=1280,3600 --screenshot="$VOSTOK_EVIDENCE/prototipo-c-tec.png" "file://$PWD/docs/estudos/documentos-finais/modelo-c/tecnico.html"`
  - Prova: o modelo C no tecnico (RF-02, RF-03, CT-02).
  - Trecho: o documento inteiro, com o conteúdo da SPEC-023 reescrito.
  - Falha: página em branco ou sem o layout do modelo.
  - Requer: `docs/estudos/documentos-finais`
- `prototipo-c-celular.png` (print): `CHROME="$(ls -d ~/.cache/ms-playwright/chromium-*/chrome-linux64/chrome | sort | tail -1)"; "$CHROME" --headless=new --no-sandbox --disable-gpu --hide-scrollbars --window-size=400,2400 --screenshot="$VOSTOK_EVIDENCE/prototipo-c-celular.png" "file://$PWD/docs/estudos/documentos-finais/modelo-c/executivo.html"`
  - Prova: o modelo C no celular, sem rolagem horizontal (RF-02, CT-02).
  - Trecho: o executivo em 400 px.
  - Falha: conteúdo cortado ou mais largo que a tela.
  - Requer: `docs/estudos/documentos-finais`
