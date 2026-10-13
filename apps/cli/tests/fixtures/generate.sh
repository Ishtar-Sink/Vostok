#!/usr/bin/env bash
# Gera as fixtures de referência (golden) do binário `vostok` a partir dos scripts bash do harness.
#
# Roda a suíte bash (`packages/harness/tests/run-tests.sh`) com cada script do harness trocado por um
# gravador. Cada chamada vira um caso em `golden/<grupo>/<NNNN>.case`, com os argumentos, o
# subcomando equivalente do `vostok`, a entrada, as saídas, o código de saída e as mudanças de arquivos.
# O formato está em `README.md`.
#
# Uso: apps/cli/tests/fixtures/generate.sh
# Requer Linux (GNU date), bash, git e python3.
set -euo pipefail

FIXTURES="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_ROOT="$(cd "$FIXTURES/../../../.." && pwd)"
GOLDEN="$FIXTURES/golden"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/vostok-golden.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

# Raiz fixa da cópia da suíte: caminhos estáveis entre execuções.
ROOT="${TMPDIR:-/tmp}/vostok-golden-root"
# Data fixa (2026-10-01T00:00:00Z) para `date` e para os commits: conteúdos e hashes estáveis.
EPOCH=1790812800

mkdir -p "$WORK/bin"
cat > "$WORK/bin/date" <<'EOF'
#!/usr/bin/env bash
# `date` com o relógio parado em GOLDEN_EPOCH, a menos que a chamada já indique a data.
for arg in "$@"; do
  case "$arg" in -d|--date|--date=*|-r|--reference|--reference=*) exec /bin/date "$@" ;; esac
done
exec /bin/date -d "@$GOLDEN_EPOCH" "$@"
EOF
chmod +x "$WORK/bin/date"

cat > "$WORK/record.py" <<'EOF'
"""Gravador: executa o script real, grava o caso e devolve as saídas a quem chamou."""
import hashlib, json, os, re, subprocess, sys

ROOT = os.environ["GOLDEN_ROOT"]
GOLDEN = os.environ["GOLDEN_DIR"]
STATE = os.environ["GOLDEN_STATE"]
SKIP = {".out"}

def blob(data):
    sha = hashlib.sha256(data).hexdigest()
    path = os.path.join(GOLDEN, "blobs", sha)
    if not os.path.exists(path):
        with open(path, "wb") as handle:
            handle.write(data)
    return sha

def git_index(repo):
    out = subprocess.run(["git", "-C", repo, "ls-files", "-s", "-z"], capture_output=True)
    return out.stdout if out.returncode == 0 else b""

def snapshot():
    files = {}
    for base, dirs, names in os.walk(ROOT):
        dirs.sort()
        rel_base = os.path.relpath(base, ROOT)
        for name in sorted(names):
            rel = os.path.normpath(os.path.join(rel_base, name))
            if rel in SKIP or rel.endswith(".real") or rel.endswith("/.git/index") or rel == ".git/index":
                continue
            full = os.path.join(base, name)
            if os.path.islink(full) or not os.path.isfile(full):
                continue
            with open(full, "rb") as handle:
                files[rel] = blob(handle.read())
        if base != ROOT and not os.access(base, os.W_OK):
            # Pasta sem permissão de escrita: entra com "/" no fim e o conteúdo "readonly".
            files[os.path.normpath(rel_base) + "/"] = blob(b"readonly\n")
        elif not dirs and not names and base != ROOT:
            # Pasta vazia: entra com "/" no fim, para comandos que olham só a existência dela.
            files[os.path.normpath(rel_base) + "/"] = blob(b"")
        if ".git" in dirs and os.path.isdir(os.path.join(base, ".git")):
            rel = os.path.normpath(os.path.join(rel_base, ".git/index.ls"))
            files[rel] = blob(git_index(base))
    return files

def diff(before, after):
    lines = [f"W {sha} {path}" for path, sha in sorted(after.items()) if before.get(path) != sha]
    lines += [f"D {path}" for path in sorted(before) if path not in after]
    return "\n".join(lines)

def normalize(text):
    text = text.replace(ROOT, "<ROOT>")
    text = re.sub(r"/tmp/tmp\.[A-Za-z0-9]+", "<TMP>", text)
    text = re.sub(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}(\.\d+)? [-+]\d{4}", "<MTIME>", text)
    return text

GROUPS = {
    "load-context.sh": "context", "load-extra-context.sh": "context",
    "add-memory.sh": "memory", "search-memory.sh": "memory", "update-memory.sh": "memory",
    "validate-rules.sh": "rules", "init-harness.sh": "rules", "adjust-rules.sh": "rules",
    "create-new-spec.sh": "spec", "list-specs.sh": "spec", "create-refined-spec.sh": "spec",
    "adjust-spec.sh": "spec", "approve-spec.sh": "spec",
    "plan-tasks.sh": "plan", "adjust-plan.sh": "plan", "approve-plan.sh": "plan",
    "load-task.sh": "task", "check-done.sh": "task", "approve-task.sh": "task",
    "adjust-task.sh": "task", "execute-task.sh": "task", "finish-task.sh": "task",
    "check-commits.sh": "commit", "approve-commit.sh": "commit",
    "create-skill.sh": "skill", "finalize-spec.sh": "finalize",
}

def flag_map(args, table, prefix):
    if args and args[0] in table:
        return prefix + table[args[0]] + args[1:]
    return None

def vostok_args(script, args):
    """Subcomando equivalente do `vostok`, ou None quando a chamada é um erro de uso."""
    simple = {"load-context.sh": ["context", "load"], "check-commits.sh": ["commit", "check"],
              "validate-rules.sh": ["rules", "validate"], "create-new-spec.sh": ["spec", "new"],
              "list-specs.sh": ["spec", "list"], "load-task.sh": ["task", "load"],
              "check-done.sh": ["task", "check-done"], "update-memory.sh": ["memory", "update"]}
    if script in simple:
        return simple[script] + args
    if script == "load-extra-context.sh":
        return ["context", "load"] + args if args else None
    if script == "add-memory.sh":
        return ["memory", "init"] + args[1:] if args[:1] == ["--init"] else ["memory", "add"] + args
    if script == "search-memory.sh":
        return ["memory", "search"] + args
    if script == "approve-commit.sh":
        return flag_map(args, {"--show": ["show"], "--approve": ["approve"], "--revert": ["revert"], "--commit": ["commit"]}, ["commit"])
    if script in ("init-harness.sh", "adjust-rules.sh"):
        table = {f"--{name}": [name] for name in ("detect", "write", "preview", "load", "unload", "next-id", "remove", "create-category")}
        return flag_map(args, table, ["rules"])
    if script == "create-refined-spec.sh":
        mapped = flag_map(args, {"--validate": ["validate"], "--mark-refined": ["mark-refined"]}, ["spec"])
        return mapped or (["spec", "refine"] + args)
    adjust = {"--record-manual": ["record-manual"], "--validate": ["validate"], "--record": ["record"]}
    if script == "adjust-spec.sh":
        return flag_map(args, adjust, ["spec", "adjust"])
    if script == "adjust-plan.sh":
        return flag_map(args, adjust, ["plan", "adjust"])
    if script == "adjust-task.sh":
        return flag_map(args, adjust, ["task", "adjust"])
    if script == "approve-spec.sh":
        return flag_map(args, {"--validate": ["validate"], "--approve": ["approve"]}, ["spec", "approve"])
    if script == "plan-tasks.sh":
        return flag_map(args, {"--create": ["create"], "--score": ["score"], "--validate": ["validate"]}, ["plan"])
    if script == "approve-plan.sh":
        return flag_map(args, {"--create-tasks": ["create-tasks"], "--validate": ["validate"], "--mark-planned": ["mark-planned"]}, ["plan", "approve"])
    if script == "approve-task.sh":
        return flag_map(args, {"--validate": ["validate"], "--approve": ["approve"]}, ["task", "approve"])
    if script == "execute-task.sh":
        return flag_map(args, {"--start": ["start"], "--baseline": ["baseline"]}, ["task"])
    if script == "finish-task.sh":
        table = {f"--{name}": [name] for name in ("tests", "regression", "evidence", "finish", "resume")}
        return flag_map(args, table, ["task"])
    if script == "create-skill.sh":
        mapped = flag_map(args, {"--script": ["script"], "--finalize": ["finalize"]}, ["skill"])
        return mapped or (["skill", "create"] + args)
    if script == "finalize-spec.sh":
        table = {f"--{name}": [name] for name in ("sources", "create", "validate", "build", "archive")}
        return flag_map(args, table, ["finalize"])
    return None

def section(name, data):
    if isinstance(data, str):
        data = data.encode()
    return b"@" + name.encode() + b" " + str(len(data)).encode() + b"\n" + data + b"\n"

def main():
    wrapper, args = sys.argv[1], sys.argv[2:]
    real = wrapper + ".real"
    if os.environ.get("GOLDEN_DEPTH"):
        os.execv(real, [real] + args)
    stdin = b"" if sys.stdin.isatty() else sys.stdin.buffer.read()
    with open(STATE) as handle:
        state = json.load(handle)
    before = snapshot()
    env = dict(os.environ, GOLDEN_DEPTH="1")
    result = subprocess.run([real] + args, input=stdin, capture_output=True, env=env)
    after = snapshot()
    sys.stdout.buffer.write(result.stdout)
    sys.stderr.buffer.write(result.stderr)

    script = os.path.basename(wrapper)
    group = GROUPS[script]
    number = state["next"]
    state["next"] += 1
    mapped = vostok_args(script, args)
    usage = mapped is None or (result.returncode != 0 and result.stderr.startswith(b"Uso:"))
    out = section("script", script)
    out += section("group", group)
    out += section("usage", "1" if usage else "0")
    for index, arg in enumerate(args):
        if not arg.startswith(ROOT) and os.path.isabs(arg) and os.path.isfile(arg):
            with open(arg, "rb") as handle:
                out += section(f"argfile.{index}", blob(handle.read()))
        out += section("arg", normalize(arg))
    for arg in mapped or []:
        out += section("vostok", normalize(arg))
    cwd = os.getcwd()
    out += section("cwd", normalize(cwd) if cwd.startswith(ROOT) else "<OUTSIDE>")
    out += section("stdin", normalize(stdin.decode(errors="replace")))
    out += section("stdout", normalize(result.stdout.decode(errors="replace")))
    out += section("stderr", normalize(result.stderr.decode(errors="replace")))
    out += section("exit", str(result.returncode))
    out += section("pre", diff(state["files"], before))
    out += section("post", diff(before, after))
    os.makedirs(os.path.join(GOLDEN, group), exist_ok=True)
    with open(os.path.join(GOLDEN, group, f"{number:04d}.case"), "wb") as handle:
        handle.write(out)
    state["files"] = after
    with open(STATE, "w") as handle:
        json.dump(state, handle)
    sys.exit(result.returncode)

if __name__ == "__main__":
    if sys.argv[1] == "--init":
        files = snapshot()
        with open(os.path.join(GOLDEN, "initial.state"), "w") as handle:
            handle.write("\n".join(f"W {sha} {path}" for path, sha in sorted(files.items())) + "\n")
        with open(STATE, "w") as handle:
            json.dump({"next": 1, "files": files}, handle)
    else:
        main()
EOF

# Instalado pela suíte depois da montagem da cópia: troca cada script por um gravador.
cat > "$WORK/hook.sh" <<'EOF'
while IFS= read -r script; do
  mv "$script" "$script.real"
  # O código original vai comentado no fim, para as buscas da suíte no conteúdo dos scripts.
  { printf '#!/usr/bin/env bash\nexec python3 "$GOLDEN_RECORD" "$0" "$@"\n'; sed 's/^/# /' "$script.real"; } > "$script"
  chmod +x "$script" "$script.real"
done < <(find "$H/scripts/default" "$H/skills" -name '*.sh' -path '*scripts*' | sort)
python3 "$GOLDEN_RECORD" --init
EOF

# Suíte com a raiz fixa e o gancho logo depois da montagem da cópia.
sed -e "s|^R=\"\$(mktemp -d .*|R=\"$ROOT\"; rm -rf \"\$R\"; mkdir -p \"\$R\"|" \
    -e 's|^# --- Contexto ---$|source "$GOLDEN_HOOK"\n&|' \
    "$SOURCE_ROOT/packages/harness/tests/run-tests.sh" > "$WORK/run-tests.sh"
grep -q 'source "$GOLDEN_HOOK"' "$WORK/run-tests.sh" || { echo "Erro: âncora da suíte não encontrada" >&2; exit 1; }

rm -rf "$GOLDEN"
mkdir -p "$GOLDEN/blobs"
export GOLDEN_ROOT="$ROOT" GOLDEN_DIR="$GOLDEN" GOLDEN_STATE="$WORK/state.json"
export GOLDEN_RECORD="$WORK/record.py" GOLDEN_HOOK="$WORK/hook.sh" GOLDEN_EPOCH="$EPOCH" PATH="$WORK/bin:$PATH"
export GIT_AUTHOR_DATE="@$EPOCH +0000" GIT_COMMITTER_DATE="@$EPOCH +0000"
# Packs do git variam entre execuções (ordem das threads): uma thread, e o remoto guarda objetos soltos.
export GIT_CONFIG_COUNT=2 GIT_CONFIG_KEY_0=pack.threads GIT_CONFIG_VALUE_0=1
export GIT_CONFIG_KEY_1=transfer.unpackLimit GIT_CONFIG_VALUE_1=100000
# A suíte fica em `packages/harness/tests`: roda a cópia de lá para resolver os caminhos de origem.
cp "$WORK/run-tests.sh" "$SOURCE_ROOT/packages/harness/tests/.run-tests-golden.sh"
trap 'rm -rf "$WORK" "$SOURCE_ROOT/packages/harness/tests/.run-tests-golden.sh" "$ROOT"' EXIT
bash "$SOURCE_ROOT/packages/harness/tests/.run-tests-golden.sh" < /dev/null > "$WORK/suite.out" || { grep -A8 "^FALHOU" "$WORK/suite.out" >&2; tail -n 1 "$WORK/suite.out" >&2; exit 1; }
tail -n 1 "$WORK/suite.out"

for dir in "$GOLDEN"/*/; do
  name="$(basename "$dir")"
  [ "$name" = blobs ] || echo "$name: $(ls "$dir" | wc -l) casos"
done
