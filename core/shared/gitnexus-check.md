<!--
  Shared GitNexus check. Dev-track workflows (`/compass:spec`, `/compass:prepare`,
  `/compass:cook`, `/compass:fix`) include this after Step 0a to detect whether
  a GitNexus index is available. Sets `$GITNEXUS_STATUS` and `$GITNEXUS_REPO`
  for downstream worker prompts and research steps.
-->

# Shared: GitNexus Check

## Check index status

```bash
GITNEXUS_STATUS="GITNEXUS_UNAVAILABLE"

if [ ! -d ".gitnexus" ]; then
  GITNEXUS_STATUS="GITNEXUS_MISSING"
elif [ -f ".gitnexus/.outdated" ]; then
  # Validate JSON + count changed_files with a real timeout.
  # Any failure (bad JSON, missing key, python hang) falls through to AVAILABLE
  # rather than silently misreporting OUTDATED.
  CHANGED=$(timeout 5s python3 -c '
import sys, json
try:
  d = json.load(sys.stdin)
  if not isinstance(d, dict):
    print("PARSE_ERROR"); sys.exit(0)
  cf = d.get("changed_files", [])
  print(len(cf) if isinstance(cf, list) else "PARSE_ERROR")
except Exception:
  print("PARSE_ERROR")
' < .gitnexus/.outdated 2>/dev/null)

  case "$CHANGED" in
    PARSE_ERROR|"")
      echo "⚠ .gitnexus/.outdated is corrupt or unreadable — treating as available; run 'npx gitnexus analyze' to refresh if this is wrong." >&2
      GITNEXUS_STATUS="GITNEXUS_AVAILABLE"
      ;;
    0)
      GITNEXUS_STATUS="GITNEXUS_AVAILABLE"
      ;;
    *)
      GITNEXUS_STATUS="GITNEXUS_OUTDATED"
      ;;
  esac
else
  GITNEXUS_STATUS="GITNEXUS_AVAILABLE"
fi

echo "$GITNEXUS_STATUS"
```

Store as `$GITNEXUS_STATUS`.

## Resolve repo name (when available)

```bash
GITNEXUS_REPO=$(timeout 5s python3 -c '
import sys, json, os
try:
  m = json.load(sys.stdin)
  print(os.path.basename(m.get("repoPath", "")))
except Exception:
  pass
' < .gitnexus/meta.json 2>/dev/null)

# Sanitize — if parsing failed or yields invalid chars, fall back to cwd basename
if ! [[ "$GITNEXUS_REPO" =~ ^[a-zA-Z0-9_-]+$ ]]; then
  GITNEXUS_REPO=$(basename "$(pwd)")
fi
```

## Branch

- **`GITNEXUS_AVAILABLE`** → store `$GITNEXUS_STATUS` + `$GITNEXUS_REPO`. Pass both to all Agent/worker prompts so they can call `gitnexus_context()` and `gitnexus_impact()`.
- **`GITNEXUS_MISSING`** or **`GITNEXUS_OUTDATED`** → AskUserQuestion (AI translates per `$LANG` — see ux-rules Language Policy):

```json
{"questions": [{"question": "GitNexus index is outdated/missing. Sync for better code intelligence?", "header": "GitNexus", "multiSelect": false, "options": [
  {"label": "Sync now", "description": "Run gitnexus analyze (~30s) — enables impact analysis, call graph, blast radius checks"},
  {"label": "Skip", "description": "Use Grep/Glob fallback — still works but may miss upstream callers"}
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
