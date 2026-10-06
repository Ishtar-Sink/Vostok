# Finalização de specs: conferência, fontes, conteúdo, documentos e arquivamento (finalize-spec.sh).
# Status vêm de config/specs.yml e config/tasks.yml; caminhos, de `paths` em config/main.yml.
# Uso: source "$HARNESS_DIR/scripts/lib/finalize.sh"
#      (requer HARNESS_DIR e as libs config.sh, text.sh, frontmatter.sh, markdown.sh, spec-validation.sh
#      e plan.sh)

# Pasta das tarefas de uma spec (`paths.tasks`/<SPEC_ID>).
# finalize_tasks_dir <SPEC_ID>
finalize_tasks_dir() {
  printf '%s/%s' "$(harness_path "$(config_get "$HARNESS_DIR/config/main.yml" paths.tasks)")" "$1"
}

# Imprime um problema por linha se a spec não puder ser finalizada: status diferente de
# `status.implemented` ou tarefa fora de `status.done`. Não imprime nada se ela puder.
# finalize_check <arquivo da spec> <pasta das tarefas da spec>
finalize_check() {
  local spec="$1" tasks_dir="$2" implemented done status task found=0
  implemented="$(config_get "$HARNESS_DIR/config/specs.yml" status.implemented)"
  done="$(config_get "$HARNESS_DIR/config/tasks.yml" status.done)"

  status="$(frontmatter_get "$spec" status 2>/dev/null || true)"
  [ "$status" = "$implemented" ] ||
    echo "a spec está com status '$status'; só specs com status '$implemented' são finalizadas"

  for task in "$tasks_dir"/*.md; do
    [ -f "$task" ] || continue
    found=1
    status="$(frontmatter_get "$task" status 2>/dev/null || true)"
    [ "$status" = "$done" ] ||
      echo "$(frontmatter_get "$task" id 2>/dev/null || basename "$task" .md): status '$status', não '$done'"
  done
  [ "$found" -eq 1 ] || echo "nenhuma tarefa encontrada em $tasks_dir"
}
