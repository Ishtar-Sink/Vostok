# Fixtures de equivalência

Saídas de referência dos antigos scripts bash do harness. Os testes de integração do `vostok` reproduzem cada caso com o subcomando equivalente e comparam saída, código de saída e arquivos (SPEC-005, CT-10 e CT-12).

## Origem

As fixtures estão congeladas: os scripts bash e a suíte deles saíram do harness na SPEC-005 (TASK-11). A última geração é a do commit `6452e65`, com o gerador `apps/cli/tests/fixtures/generate.sh` daquele commit (`git show 6452e65:apps/cli/tests/fixtures/generate.sh`), que rodava a suíte bash com cada script trocado por um gravador. Cada chamada a um script virou um caso.

A geração era estável. Para isso, o gerador:

- usava uma raiz fixa para a cópia da suíte (`$TMPDIR/vostok-golden-root`);
- parava o relógio em 2026-10-01T00:00:00Z, com um `date` próprio no `PATH` e as datas de commit do git fixas;
- trocava nos textos gravados a raiz por `<ROOT>`, os temporários do `mktemp` por `<TMP>` e as datas de modificação do `diff` por `<MTIME>`.

Uma mudança de comportamento do `vostok` que altere um caso exige editar a fixture do caso (e o blob dela), com a justificativa no commit.

## Estrutura

```text
golden/
  initial.state       # estado da cópia antes do primeiro caso
  blobs/<sha256>      # conteúdo de cada arquivo, pelo SHA-256
  <grupo>/<NNNN>.case # um caso por chamada; NNNN é a ordem global de execução
```

Os grupos seguem os do `vostok`: `context`, `memory`, `rules`, `spec`, `plan`, `task`, `commit`, `skill` e `finalize`.

## Formato do caso

Cada seção é `@<nome> <tamanho em bytes>`, uma quebra de linha, o conteúdo e outra quebra de linha. Seções repetidas aparecem na ordem.

| Seção | Conteúdo |
|---|---|
| `script` | Nome do script bash chamado. |
| `group` | Grupo do `vostok`. |
| `usage` | `1` quando a chamada é um erro de uso (sem subcomando equivalente ou com saída `Uso:`); o teste só confere que o `vostok` falha. |
| `arg` | Um argumento do script, repetida para cada um. |
| `argfile.<i>` | SHA-256 do arquivo de fora da raiz passado no argumento `i` (ex.: arquivo de regras temporário). O teste grava o blob num temporário e troca o argumento. |
| `vostok` | Um argumento do subcomando equivalente, repetida para cada um (ex.: `spec`, `approve`, `validate`, `SPEC-001`). |
| `cwd` | Diretório da chamada, com `<ROOT>`, ou `<OUTSIDE>` quando fora da raiz. |
| `stdin` | Entrada padrão (ex.: mensagem de commit). |
| `stdout`, `stderr` | Saídas, normalizadas. |
| `exit` | Código de saída. |
| `pre` | Mudanças feitas pela própria suíte desde o caso anterior, antes desta chamada. |
| `post` | Mudanças feitas por esta chamada. |

`pre` e `post` têm uma linha por arquivo: `W <sha256> <caminho>` (gravado com o conteúdo do blob) ou `D <caminho>` (removido). Os caminhos são relativos à raiz. Uma pasta vazia aparece com `/` no fim do caminho e conteúdo vazio; uma pasta sem permissão de escrita, com `/` no fim e o conteúdo `readonly`.

## Como reproduzir um caso

1. Parta de `initial.state` e aplique, em ordem global (`NNNN`), o `pre` e o `post` de todos os casos anteriores, de qualquer grupo. Isso dá o estado exato antes do caso.
2. Aplique o `pre` do caso, materialize o estado numa pasta temporária e rode `vostok` com as seções `vostok`, a partir da raiz (ou de `<ROOT>/<cwd>`).
3. Compare `stdout`, `stderr` e `exit` e o efeito sobre os arquivos com o `post`, normalizando nas duas pontas a raiz, datas (`AAAA-MM-DD`, `AAAA-MM-DDTHH:MM:SSZ`) e hashes (SHA-256 e hashes de commit), que mudam com o relógio.

## Limitações

- O índice do git não é gravado como binário: `.git/index.ls` guarda a saída de `git ls-files -s -z`, e o teste recria o índice com `git update-index --index-info`.
- Os scripts originais e os gravadores (`*.real`) ficam fora dos estados.
- `init-harness.sh --load` e `adjust-rules.sh --load` viram o mesmo `vostok rules load` (o primeiro delega ao segundo); a seção `script` indica qual deles gerou o caso.
- Chamadas aninhadas (um script chamando outro) contam como um caso só, o de fora.
