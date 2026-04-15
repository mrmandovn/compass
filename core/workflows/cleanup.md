# Workflow: compass:cleanup

You are the housekeeper. Mission: keep the Compass project tidy by closing stale pipelines, archiving old sessions, and — only with explicit confirmation — purging archived sessions older than the retention window.

**Principles:** Safety first. Never delete without preview + explicit confirmation. Archive is reversible; purge is not. Always show the PO exactly what will change before applying.

**Purpose**: Provide a single entry point for session / pipeline housekeeping in the current project.

**Output**: Pipeline status updates + optional folder moves (`sessions/<slug>/` → `sessions/_archived/<slug>/`) + optional hard deletes of archived items.

**When to use**:
- After several briefs/runs when the `sessions/` directory feels cluttered
- When `/compass:prd` keeps surfacing an old forgotten pipeline you'd rather close
- As a scheduled quarterly cleanup

---

Apply the UX rules from `core/shared/ux-rules.md`.

> **Additional rule for cleanup**: this workflow is exempt from `core/shared/resolve-project.md` Step 0d (pipeline+project gate) — cleanup IS the housekeeping for pipelines, so applying the gate to cleanup itself would be circular.

---

## Step 0 — Resolve active project

Apply Step 0 + Step 0a–0c from `core/shared/resolve-project.md` (get `$PROJECT_ROOT`, `$CONFIG`, `$PROJECT_NAME`, `$SHARED_ROOT`). **SKIP Step 0d** — see rule above.

From `$CONFIG`, extract `lang`. All user-facing output from here is in `$LANG`.

---

## Step 0a — Parse subcommand

`$ARGUMENTS` routes the workflow. Parse once:

```bash
ARG="${ARGUMENTS:-}"
DRY_RUN="false"
FORCE="false"
case "$ARG" in
  "")
    MODE="interactive"
    ;;
  *--dry-run*) DRY_RUN="true"; ARG=$(echo "$ARG" | sed 's/--dry-run//g' | xargs) ;;
esac
case "$ARG" in
  *--confirm*) FORCE="true"; ARG=$(echo "$ARG" | sed 's/--confirm//g' | xargs) ;;
esac
case "$ARG" in
  "")                           MODE="${MODE:-interactive}" ;;
  "--stale")                    MODE="close-stale" ;;
  "--archive")                  MODE="archive-old-completed" ;;
  "--purge")                    MODE="purge-archived" ;;
  "--list"|"--inventory")       MODE="inventory" ;;
  *)                            MODE="usage" ;;
esac
echo "MODE=$MODE DRY_RUN=$DRY_RUN FORCE=$FORCE"
```

**Usage message** (`MODE=usage`):

```
Usage:
  /compass:cleanup                      Interactive housekeeping
  /compass:cleanup --inventory          Show counts of active / completed / archived
  /compass:cleanup --stale              Close active pipelines >14d with 0 artifacts
  /compass:cleanup --archive            Move completed sessions >30d to sessions/_archived/
  /compass:cleanup --purge --confirm    Hard delete archived sessions >90d (destructive)
  /compass:cleanup --dry-run <action>   Preview without applying
```

Print and stop.

---

## Step 1 — Inventory

Always compute the inventory first, regardless of `$MODE` (interactive needs it; non-interactive actions use it for filtering):

```bash
SESSIONS_DIR="$PROJECT_ROOT/.compass/.state/sessions"
ARCHIVED_DIR="$SESSIONS_DIR/_archived"
NOW=$(date -u +%s)

ACTIVE_FRESH=()
ACTIVE_STALE=()
COMPLETED_RECENT=()
COMPLETED_OLD=()
ARCHIVED_RECENT=()
ARCHIVED_OLD=()

for PF in $(find "$SESSIONS_DIR" -maxdepth 3 -name "pipeline.json" 2>/dev/null); do
  SLUG=$(basename "$(dirname "$PF")")
  STATUS=$(jq -r '.status' "$PF" 2>/dev/null)
  CREATED=$(jq -r '.created_at' "$PF" 2>/dev/null)
  CLOSED=$(jq -r '.completed_at // empty' "$PF" 2>/dev/null)
  ART_COUNT=$(jq -r '.artifacts | length' "$PF" 2>/dev/null || echo 0)
  CREATED_SEC=$(date -j -f "%Y-%m-%dT%H:%M:%SZ" "$CREATED" +%s 2>/dev/null || echo "$NOW")
  AGE_DAYS=$(( (NOW - CREATED_SEC) / 86400 ))
  # Is this session under _archived/?
  IS_ARCHIVED=$(echo "$PF" | grep -q "/_archived/" && echo yes || echo no)

  if [ "$IS_ARCHIVED" = "yes" ]; then
    if [ "$AGE_DAYS" -gt 90 ]; then ARCHIVED_OLD+=("$SLUG|$AGE_DAYS|$ART_COUNT")
    else ARCHIVED_RECENT+=("$SLUG|$AGE_DAYS|$ART_COUNT"); fi
  elif [ "$STATUS" = "active" ]; then
    if [ "$AGE_DAYS" -gt 14 ] && [ "$ART_COUNT" = "0" ]; then ACTIVE_STALE+=("$SLUG|$AGE_DAYS|$ART_COUNT")
    else ACTIVE_FRESH+=("$SLUG|$AGE_DAYS|$ART_COUNT"); fi
  elif [ "$STATUS" = "completed" ]; then
    CLOSED_SEC=$(date -j -f "%Y-%m-%dT%H:%M:%SZ" "$CLOSED" +%s 2>/dev/null || echo "$NOW")
    AGE_CLOSED=$(( (NOW - CLOSED_SEC) / 86400 ))
    if [ "$AGE_CLOSED" -gt 30 ]; then COMPLETED_OLD+=("$SLUG|$AGE_CLOSED|$ART_COUNT")
    else COMPLETED_RECENT+=("$SLUG|$AGE_CLOSED|$ART_COUNT"); fi
  fi
done
```

### Step 1a — Print inventory summary (always)

en:
```
📋 Session inventory — <PROJECT_NAME>

  Active pipelines:        <#ACTIVE_FRESH> fresh, <#ACTIVE_STALE> stale (⚠ >14d + 0 artifacts)
  Completed sessions:      <#COMPLETED_RECENT> recent (≤30d), <#COMPLETED_OLD> old (>30d)
  Archived sessions:       <#ARCHIVED_RECENT> recent (≤90d), <#ARCHIVED_OLD> old (>90d) — eligible for purge
```

vi: equivalent structure, translated labels.

If `MODE=inventory`, stop here.

---

## Step 2 — Apply mode

### MODE=interactive

Ask via AskUserQuestion:

en:
```json
{"questions": [{"question": "What should the cleanup do?", "header": "Cleanup", "multiSelect": false, "options": [
  {"label": "Close stale pipelines", "description": "Mark <#ACTIVE_STALE> active pipelines as completed (active >14d, 0 artifacts)"},
  {"label": "Archive old completed sessions", "description": "Move <#COMPLETED_OLD> sessions (completed >30d) to sessions/_archived/"},
  {"label": "Purge archived > 90d", "description": "Delete <#ARCHIVED_OLD> archived sessions permanently (irreversible)"},
  {"label": "Show details", "description": "List every session in each bucket"}
]}]}
```

vi: equivalent.

**Note:** if a bucket is empty (e.g. `#ACTIVE_STALE == 0`), omit that option from the list. If ALL buckets are empty → print `✓ Nothing to clean up.` and stop.

Route to the matching sub-mode below after the pick.

### MODE=close-stale

- Sub-step: for each slug in `ACTIVE_STALE`, plan `pipeline.json` status update → `completed`, stamp `completed_at=now`, and add `completed_reason="auto-closed by /compass:cleanup --stale (>14d, 0 artifacts)"`.
- If `DRY_RUN=true` → print each planned change and stop. Otherwise:
- Show a single confirmation AskUserQuestion listing all affected slugs.
- On confirm, apply with atomic jq + mv pattern:
  ```bash
  TMP=$(mktemp)
  jq --arg t "$(date -u +%Y-%m-%dT%H:%M:%SZ)" '.status = "completed" | .completed_at = $t | .completed_reason = "auto-closed by /compass:cleanup --stale"' "$PF" > "$TMP" && mv "$TMP" "$PF"
  ```
- Print final count `✓ Closed <N> stale pipelines`.

### MODE=archive-old-completed

- For each slug in `COMPLETED_OLD`, plan: move `$SESSIONS_DIR/<slug>/` → `$ARCHIVED_DIR/<slug>/`.
- Create `$ARCHIVED_DIR` if missing (`mkdir -p`).
- If `DRY_RUN=true` → print planned moves and stop. Otherwise:
- Show confirmation AskUserQuestion listing all affected slugs + destination path.
- On confirm, apply: `mv "$SESSIONS_DIR/<slug>" "$ARCHIVED_DIR/<slug>"`.
- Print final count `✓ Archived <N> completed sessions to $ARCHIVED_DIR`.
- **Reversible** — note in the success message: `Revert with: mv $ARCHIVED_DIR/<slug> $SESSIONS_DIR/<slug>`.

### MODE=purge-archived

- Safety: require `FORCE=true` (i.e. `--confirm` flag on CLI) OR an explicit PO confirmation in interactive mode. If neither is set, print:
  ```
  ⚠ Purge is irreversible. Re-run with --purge --confirm, or pick "Purge" in interactive mode (which will re-confirm).
  ```
  and stop.
- For each slug in `ARCHIVED_OLD`, plan: `rm -rf $ARCHIVED_DIR/<slug>`.
- Always print the slugs that will be purged and the total disk space that will be reclaimed (best-effort `du -sh`).
- If `DRY_RUN=true` → stop here with the preview.
- Otherwise, show a **final** AskUserQuestion (even when `--confirm` was passed) restating "This will permanently delete N sessions. Type the word `DELETE` (or pick `Confirm permanent delete`) to proceed."
  - In bash-only contexts, substitute the `Type your own answer` affordance with a strict literal match check.
- On the explicit confirmation only, execute the `rm -rf` loop.
- Print final count `✓ Purged <N> archived sessions`.

### MODE=inventory

Already handled in Step 1a — nothing more to do. Stop.

### MODE=usage

Already handled in Step 0a — nothing more to do. Stop.

---

## Step 3 — Post-action summary

After any non-inventory mode completes (interactive or flag-driven), print a final line:

en:
```
✓ Cleanup done. Next: /compass:cleanup --inventory to re-check state.
```

vi:
```
✓ Cleanup xong. Tiếp: /compass:cleanup --inventory để check lại state.
```

---

## Edge cases

| Situation | Handling |
|---|---|
| `sessions/` directory missing | Print `ℹ No sessions directory yet. Run /compass:brief to create one.` and stop. |
| `_archived/` contains files without `pipeline.json` | Skip them silently (never delete anything that doesn't look like a Compass session). |
| Concurrent write (another /compass:* running) | Use atomic write pattern (mktemp + mv); do NOT lock. If jq parse fails, skip the file and continue. |
| Clock skew (created_at > now) | Treat `AGE_DAYS = 0`, categorize as fresh. |
| DRY_RUN flag combined with interactive | Honor it — preview every action, never apply. |
| PO cancels mid-confirm | Abort cleanly, no partial updates. |
| Permission error on archive/purge | Surface the error with the slug and continue to the next item — do NOT abort the whole batch. |

---

## Final — Hand-off

After cleanup completes (or was a no-op), print one of:

- en: `✓ Cleanup complete. Next: /compass:brief to start a fresh task, or /compass:status to view project health.`
- vi: `✓ Cleanup xong. Tiếp: /compass:brief để bắt đầu task mới, hoặc /compass:status để xem project health.`

Then stop. Do NOT auto-invoke the next workflow.
