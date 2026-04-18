<!--
  Shared GitNexus check. Dev-track workflows (`/compass:spec`, `/compass:prepare`,
  `/compass:cook`, `/compass:fix`) include this after Step 0a to detect whether
  a GitNexus index is available. Sets `$GITNEXUS_STATUS` and `$GITNEXUS_REPO`
  for downstream worker prompts and research steps.
-->

# Shared: GitNexus Check

## Check index status

```bash
if [ ! -d ".gitnexus" ]; then
  echo "GITNEXUS_MISSING"
elif [ -f ".gitnexus/.outdated" ] && [ "$(cat .gitnexus/.outdated 2>/dev/null | python3 -c 'import sys,json; d=json.load(sys.stdin); print(len(d.get("changed_files",[])))' 2>/dev/null)" != "0" ]; then
  echo "GITNEXUS_OUTDATED"
else
  echo "GITNEXUS_AVAILABLE"
fi
```

Store as `$GITNEXUS_STATUS`.

## Resolve repo name (when available)

```bash
GITNEXUS_REPO=$(cat .gitnexus/meta.json 2>/dev/null | python3 -c 'import sys,json,os; m=json.load(sys.stdin); print(os.path.basename(m.get("repoPath","")))' 2>/dev/null || basename "$(pwd)")
[[ "$GITNEXUS_REPO" =~ ^[a-zA-Z0-9_-]+$ ]] || GITNEXUS_REPO=$(basename "$(pwd)")
```

## Branch

- **`GITNEXUS_AVAILABLE`** → store `$GITNEXUS_STATUS` + `$GITNEXUS_REPO`. Pass both to all Agent/worker prompts so they can call `gitnexus_context()` and `gitnexus_impact()`.
- **`GITNEXUS_MISSING`** or **`GITNEXUS_OUTDATED`** → AskUserQuestion:

en:
```json
{"questions": [{"question": "GitNexus index is outdated/missing. Sync for better code intelligence?", "header": "GitNexus", "multiSelect": false, "options": [
  {"label": "Sync now", "description": "Run gitnexus analyze (~30s) — enables impact analysis, call graph, blast radius checks"},
  {"label": "Skip", "description": "Use Grep/Glob fallback — still works but may miss upstream callers"}
]}]}
```

vi:
```json
{"questions": [{"question": "GitNexus index bị outdated/missing. Sync lại?", "header": "GitNexus", "multiSelect": false, "options": [
  {"label": "Sync ngay", "description": "Chạy gitnexus analyze (~30s) — bật impact analysis, call graph, blast radius"},
  {"label": "Bỏ qua", "description": "Dùng Grep/Glob — vẫn chạy được nhưng có thể miss upstream callers"}
]}]}
```

If sync: `npx gitnexus analyze --embeddings`. Set `$GITNEXUS_STATUS` = `GITNEXUS_AVAILABLE`.
If skip: Set `$GITNEXUS_STATUS` = `GITNEXUS_UNAVAILABLE`.

## Usage in worker prompts

When building Agent prompts for workers, include:
```
GitNexus: <$GITNEXUS_STATUS>
GitNexus Repo: <$GITNEXUS_REPO>
```

Workers should call `gitnexus_impact({target: "symbolName", direction: "upstream", repo: $GITNEXUS_REPO})` before modifying any symbol (if GitNexus available). HIGH/CRITICAL risk → report to orchestrator before proceeding.
