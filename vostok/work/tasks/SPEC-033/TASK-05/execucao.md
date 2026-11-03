### Ler

- `vostok/work/specs/refined/SPEC-033-operar-o-projeto-de-ponta-a-ponta-pelo-painel-v0-4-0.md` (RF-10, RF-11, RNF-03, RN-05, CB-02, CB-05, CT-11, CT-12, decisão 7)
- `docs/estudos/painel-ponta-a-ponta.md` (seção "Arquivos de comunicação": ciclo de vida com a spec, pedidos sem spec e o comando de limpeza)
- `vostok/work/tasks/SPEC-033/TASK-01/tecnico.md`, `vostok/work/tasks/SPEC-033/TASK-02/tecnico.md`, `vostok/work/tasks/SPEC-033/TASK-03/tecnico.md` e `vostok/work/tasks/SPEC-033/TASK-04/tecnico.md` (leitores de `paths`/`folders`, pastas por estado, `<approvals>/<SPEC>.log` e o destino das perguntas arquivadas)
- `apps/cli/src/commands/finalize.rs` (fn archive, fn archive_problems, fn copy_to_archive, fn archive_dir, fn copy_file, fn remove_path, struct Finalization)
- `apps/cli/src/commands/pedidos.rs` (enum PedidosCommand, fn run, a listagem por estado e as funções de pasta da TASK-02)
- `apps/cli/src/support.rs` (fn utc_datetime, fn civil_from_days)
- `apps/cli/src/harness.rs`
- `apps/cli/tests/finalize.rs` (fn built_folder_project, teste `finalize_folder_archive_moves_the_task_folders_with_their_evidence`)
- `apps/cli/tests/pedidos.rs` (fixtures de pedido e `stdout_of`)

### Mudar

- `apps/cli/src/commands/finalize.rs` (alterar): `vostok finalize archive` leva os pedidos concluídos, as perguntas e o log de avais da spec para `<archive>/<SPEC>/` (RF-10, CT-11)
- `apps/cli/src/commands/pedidos.rs` (alterar): `vostok pedidos prune [--confirmar]` e as funções públicas de limpeza, para o painel chamar a mesma (RF-11, CT-12)
- `apps/cli/src/support.rs` (alterar): `utc_datetime_days_ago`, o limite de 30 dias do `prune` sem dependência nova
- `apps/cli/tests/finalize.rs` (alterar): teste do arquivamento com pedidos, perguntas e avais
- `apps/cli/tests/pedidos.rs` (alterar): testes do `prune`

### Passos

1. `apps/cli/tests/finalize.rs` (alterar, testes): acrescente o helper `write_file(project, relative, content)` e o teste `archive_leva_pedidos_perguntas_e_avais_da_spec` (CT-11). Use `built_folder_project()` e, antes do `vostok finalize archive SPEC-001`, grave:
   - pedidos de `SPEC-001` em `vostok/work/pedidos/feito/PED-001.md`, `recusado/PED-002.md` e `em_andamento/PED-004.md`, e um de `SPEC-002` em `feito/PED-003.md`;
   - perguntas de `SPEC-001` em `vostok/work/perguntas/respondida/P-001.md` e `aberta/P-002.md`, e uma de `SPEC-002` em `aberta/P-003.md`;
   - `vostok/work/avais/SPEC-001.log` com duas linhas quaisquer e `vostok/work/avais/SPEC-002.log`.

   Depois do archive, confira:
   - PED-001 e PED-002 estão em `vostok/work/.archive/SPEC-001/pedidos/`, sem subpasta de estado, e sumiram da pasta de trabalho;
   - PED-003 continua em `feito/` e PED-004 continua em `em_andamento/`;
   - P-001 e P-002 estão em `.archive/SPEC-001/perguntas/` e P-003 ficou;
   - `.archive/SPEC-001/avais/SPEC-001.log` tem exatamente os bytes do original, que sumiu, e `SPEC-002.log` ficou;
   - a saída cita `2 pedido(s)`, `2 pergunta(s)` e `PED-004`.

   Confira: o teste falha antes do passo 3.
2. `apps/cli/tests/pedidos.rs` (alterar, testes): acrescente o helper `write_at(project, relative, front_matter)` e os testes abaixo. Nos testes, "antigo" é `atualizado_em`/`created_at` `2026-01-01T00:00:00Z` e "recente" é `2999-01-01T00:00:00Z`.
   - `prune_sem_confirmar_so_lista` (CT-12, RNF-03): a fixture tem:
     - `.archive/SPEC-001/pedidos/PED-001.md`, `.archive/SPEC-001/perguntas/P-001.md` e `.archive/SPEC-001/avais/SPEC-001.log`;
     - em `vostok/work/pedidos/`: `feito/PED-002.md` sem spec e antigo, `feito/PED-003.md` sem spec e recente, `feito/PED-004.md` com `spec: SPEC-009` e antigo, `pendente/PED-005.md` sem spec e antigo;
     - `perguntas/respondida/P-002.md` sem spec e antiga;
     - `vostok/work/avais/sem-spec.log`.

     `vostok pedidos prune` lista PED-001, P-001, PED-002 e P-002, não lista PED-003, PED-004, PED-005 nem os `.log`, cita `--confirmar`, e todos os arquivos continuam lá.
   - `prune_com_confirmar_apaga_e_mantem_os_avais` (CT-12, RNF-03): mesma fixture com `--confirmar`. Os quatro listados somem, os demais ficam, e `.archive/SPEC-001/avais/SPEC-001.log` e `work/avais/sem-spec.log` continuam.
   - `prune_leva_o_pedido_concluido_de_spec_arquivada`: `feito/PED-006.md` com `spec: SPEC-001` e recente, com `.archive/SPEC-001/` existente, é listado.
   - `prune_sem_nada_diz_que_nao_ha_o_que_limpar` (CB-05): a saída é `Não há o que limpar.` e o código é 0.

   Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test pedidos -- prune_` falha antes do passo 5.
3. `apps/cli/src/commands/finalize.rs` (alterar): acrescente a `Finalization` `fn communication_files(&self) -> Result<Communication>`, com:
   - `requests`: os `PED-NNN.md` de `SPEC_ID` nas subpastas de feito e recusado de `harness.path("requests")`;
   - `in_flight`: os IDs e estados dos de `SPEC_ID` em pendente e em andamento;
   - `questions`: os `P-NNN.md` de `SPEC_ID` nas subpastas de aberta e respondida de `harness.path("questions")`;
   - `approvals`: `harness.path("approvals")/<SPEC_ID>.log`, se existir.

   A spec de cada arquivo vem de `frontmatter_get(file, "spec")`, e os nomes das subpastas vêm do leitor de `folders` da TASK-01. Em `fn copy_to_archive`, depois das tarefas, copie com `copy_file` cada arquivo para `archive.join(<último trecho do caminho configurado>)`, isto é, `pedidos/`, `perguntas/` e `avais/` no padrão, sem subpasta de estado. Crie essas pastas com `mkdir_p` só quando houver arquivo a copiar. Em `fn archive`, depois de `remove_path(&self.content_dir()?)`, remova os originais copiados com `remove_path`. Se houve algum arquivo, imprima `Pedidos, perguntas e avais de <SPEC> levados para <archive>: N pedido(s), M pergunta(s), avais: sim|não.`. Para cada pedido em `in_flight`, imprima `<PED> (<estado>) fica na pasta de trabalho; o vostok pedidos prune o limpa depois de concluído.`. Sem nada a levar, a saída fica igual à de hoje, e o `golden_finalize` não muda. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test finalize -- archive_leva_pedidos_perguntas_e_avais_da_spec golden_finalize`.
4. `apps/cli/src/support.rs` (alterar): extraia de `fn utc_datetime` a formatação para `fn format_utc(seconds: u64) -> String` e acrescente `pub fn utc_datetime_days_ago(days: u64) -> String`, que formata `agora - days * 86_400`. `utc_datetime` passa a chamar `format_utc`. Confira: `cargo build --manifest-path apps/cli/Cargo.toml`.
5. `apps/cli/src/commands/pedidos.rs` (alterar): acrescente a variante `/// Lista os pedidos e as perguntas que podem ser apagados; só apaga com --confirmar.` `Prune { #[arg(long)] confirmar: bool }`. Acrescente também `const PRUNE_DAYS: u64 = 30;`, `pub struct PruneItem { pub path: PathBuf, pub spec: String, pub reason: String }`, `pub fn prune_candidates(harness: &Harness) -> Result<Vec<PruneItem>>` e `pub fn prune_delete(items: &[PruneItem]) -> Result<usize>`. Os candidatos, nesta ordem e ordenados por caminho dentro de cada grupo, são:
   - (a) os `.md` de `<archive>/*/<pasta de pedidos>/` e `<archive>/*/<pasta de perguntas>/`, com o motivo `arquivado`;
   - (b) na pasta de trabalho, os pedidos de feito e recusado sem `spec` com `atualizado_em` menor que `utc_datetime_days_ago(PRUNE_DAYS)`, e as perguntas respondidas sem `spec` com `answered_at`, ou `created_at` quando ele falta, menor que esse limite, com o motivo `concluído há mais de 30 dias`;
   - (c) os pedidos de feito e recusado e as perguntas respondidas cuja `spec` já tem a pasta `<archive>/<spec>/`, com o motivo `spec arquivada`.

   Nunca entram arquivos `.log` nem nada de `paths.approvals`. O `prune` imprime a tabela `| Arquivo | Spec | Motivo |`, com o caminho relativo à raiz do harness e `-` sem spec. Sem `--confirmar`, termina com `N arquivo(s), de <specs ou "sem spec">. Nada foi apagado: rode vostok pedidos prune --confirmar para apagar.`. Com `--confirmar`, chama `prune_delete`, remove as pastas `pedidos/` e `perguntas/` do arquivo que ficarem vazias e termina com `N arquivo(s) apagado(s), de <specs>; os avais ficam.`. Sem candidatos, imprime só `Não há o que limpar.`. Confira: `cargo test --manifest-path apps/cli/Cargo.toml --test pedidos -- prune_`.
6. Rode `cargo clippy --manifest-path apps/cli/Cargo.toml --all-targets -- -D warnings` e `cargo fmt --manifest-path apps/cli/Cargo.toml -- --check`. Confira: sem aviso nem diferença.
