# Workflow: compass:fix

You are the hotfix lead. Mission: take a bug description → trace root cause across layers (UI / API / data / config) → propose ≥2 hypotheses with evidence → apply a minimal fix in a single wave → verify → commit.

**Principles:** Minimal is the right size. Fix the root cause, not the symptom. Don't refactor while fixing. Propose alternatives, let the dev pick. Scope soft-check >5 files OR >1 layer → ask dev to redirect; scope hard cap >20 files → force redirect to `/compass:spec` + `/compass:prepare` + `/compass:cook` (no override).

**Purpose**: Targeted bug fix with cross-layer tracing, without going through the full spec + prepare + build loop.

**Output** (consolidated — 1 markdown + 1 state.json + 1 worker report):
- `$SESSION_DIR/FIX-PLAN.md` — single artifact: symptom + root cause + rejected alternatives (audit) + patch + verify
- `$SESSION_DIR/state.json` — metadata + `files_affected[]` array + `task_type: fix` + `is_hotfix: true` + `status` + `commit_sha`
- `$SESSION_DIR/.worker-report.json` — sub-agent return payload
- Single commit on a `fix/<slug>` branch with `fix(<scope>): <summary>` message

**When to use**:
- A bug is small enough to fit in one commit (≤5 files, single layer)
- You have an error log, stack trace, or symptom description
- You don't need a PRD-style spec for the fix

---

Apply the UX rules from `core/shared/ux-rules.md`.

---

## Step 0 — Resolve active project

Apply `core/shared/resolve-project.md`.

---

## Step 0a — Pipeline gate

Apply Step 0d. Dev sessions standalone.

---

## Step 0b — GitNexus check

Apply the shared snippet from `core/shared/gitnexus-check.md`. Sets `$GITNEXUS_STATUS` and `$GITNEXUS_REPO`. When available, use `gitnexus_impact()` in cross-layer trace (Step 4) instead of bash find/grep fallback. Include both in the Agent worker prompt (Step 10).

---

## Step 1 — Git context

Apply `core/shared/git-context.md` Parts A + B. Session slug will be derived after Step 2; branch creation happens in Step 6.

---

## Step 2 — Input parsing

`$ARGUMENTS` may be:
- Free-text bug description — "Button click not working in /login page"
- File path to error log / failing test output — read content, extract error patterns
- Empty — ask via AskUserQuestion

```json
{"questions": [{"question": "What's the bug?", "header": "Input", "multiSelect": false, "options": [
  {"label": "Describe the bug", "description": "Type the symptom / error message in Other"},
  {"label": "Paste error log path", "description": "Absolute or relative path to a log file"},
  {"label": "Paste test output", "description": "Failing test output dump (any length)"}
]}]}
```

vi: translate.

Derive title (first 60 chars), store `$BUG_DESC`.

---

## Step 3 — Triage AskUserQuestion

Ask the dev to narrow the scope — this routes the tracing logic:

en:
```json
{"questions": [{"question": "Bug scope?", "header": "Scope", "multiSelect": false, "options": [
  {"label": "UI / frontend", "description": "Component render, state, user interaction, styling"},
  {"label": "API / backend", "description": "Endpoint, service layer, database, middleware"},
  {"label": "Config / build / CI", "description": "Env vars, build scripts, CI workflows, Docker"},
  {"label": "Unclear — I'll trace", "description": "Not sure where — scan broadly (recent commits + error keywords)"}
]}]}
```

vi: translate.

Store `$SCOPE` (one of ui / api / config / unclear).

---

## Step 4 — Cross-layer trace

**If `$GITNEXUS_STATUS` = `GITNEXUS_AVAILABLE`** (preferred — replaces bash find/grep below):

For each keyword extracted from `$BUG_DESC`:
- `gitnexus_impact({target: "<keyword>", direction: "upstream", repo: "$GITNEXUS_REPO"})` → trace all callers across layers
- `gitnexus_context({name: "<keyword>", repo: "$GITNEXUS_REPO"})` → get call graph, process participation

Use impact results to identify cross-layer boundaries (e.g. frontend component calls API endpoint calls database query). Skip the bash blocks below — GitNexus provides more accurate results faster.

**Fallback (when `$GITNEXUS_STATUS` != `GITNEXUS_AVAILABLE`):**

**Source file listing (shared by all scopes)** — respects `.gitignore`, excludes noise dirs, real timeout:

```bash
# Shell helper — list candidate source files safely
list_src() {
  if git -C "$PROJECT_ROOT" rev-parse --git-dir >/dev/null 2>&1; then
    # git ls-files respects .gitignore → no node_modules / dist / build / .next
    timeout 20s git -C "$PROJECT_ROOT" ls-files 2>/dev/null
  else
    timeout 20s find "$PROJECT_ROOT" \
      \( -path '*/node_modules' -o -path '*/.git' -o -path '*/dist' \
         -o -path '*/build' -o -path '*/.next' -o -path '*/.nuxt' \
         -o -path '*/coverage' -o -path '*/vendor' -o -path '*/target' \
         -o -path '*/.venv' -o -path '*/venv' -o -path '*/__pycache__' \
         -o -path '*/.turbo' -o -path '*/.cache' \) -prune \
      -o -type f -print 2>/dev/null
  fi
}

# Keyword extraction — min 5 chars, skip English stopwords
STOP_WORDS='^(click|button|error|value|when|with|from|this|that|form|input|page|view|data|call|text|item|work|show|list|user|null|true|false|undefined)$'
KEYWORDS=$(echo "$BUG_DESC" | grep -oE '[a-zA-Z][a-zA-Z0-9_-]{4,}' | sort -u | grep -Evi "$STOP_WORDS" | head -5)
[ -z "$KEYWORDS" ] && KEYWORDS=$(echo "$BUG_DESC" | grep -oE '[a-zA-Z][a-zA-Z0-9_-]{4,}' | sort -u | head -3)

# Safe keyword grep — cap total matches per keyword at 20
kw_grep() {
  local pattern="$1"; local files; files=$(list_src)
  [ -z "$files" ] && return 0
  echo "$files" | timeout 15s xargs -I{} -P 1 grep -lE "$pattern" -- "{}" 2>/dev/null | head -20
}
```

Per `$SCOPE`:

**UI**:
```bash
UI_PATHS=$(list_src | grep -E '(^|/)(components|pages|app|ui|views|screens|routes)/' | head -200)
for kw in $KEYWORDS; do
  echo "$UI_PATHS" | timeout 10s xargs -I{} grep -l -- "$kw" "{}" 2>/dev/null | head -10
done
kw_grep 'useState|useReducer|useStore|defineStore|createStore|createSignal'
kw_grep 'fetch\(|axios\.|api\.|http\.(get|post|put|delete)'
```

**API**:
```bash
kw_grep 'router\.|app\.(get|post|put|delete|patch)|@(Route|Controller|Get|Post|Put|Delete)'
list_src | grep -E '(^|/)(services|handlers|controllers|routes|api)/' | head -100
kw_grep 'middleware|authenticate|authorize|requireAuth'
```

**Config**:
```bash
list_src | grep -E '(^|/)(\.env[^/]*|[^/]*\.config\.(js|ts|mjs|cjs|json|yaml|yml)|docker-compose[^/]*\.ya?ml|Dockerfile[^/]*|Makefile|build\.sh|package\.json|pyproject\.toml|Cargo\.toml|go\.mod)$' | head -80
list_src | grep -E '^(\.github/workflows|\.gitlab-ci|\.circleci|\.buildkite)' | head -40
```

**Unclear**:
```bash
# Recent commits touching tracked files
timeout 15s git -C "$PROJECT_ROOT" log --since="30.days.ago" --name-only --pretty=format:"%h %s" 2>/dev/null | head -80

# Keyword scan limited to source files
for kw in $KEYWORDS; do
  echo "  ↳ keyword: $kw"
  kw_grep "$kw" | head -5
done
```

**Timeout + soft-fail**: every `timeout` command above is real (not a comment). If any `timeout` returns exit code 124, print `ℹ Trace for <scope> timed out — results partial.` and continue. Never let trace hang the workflow.

**Never fabricate evidence**: if trace produced 0 meaningful hits, Step 5 MUST state `Evidence: none — trace returned no matches` on at least one hypothesis, rather than inventing paths.

---

## Step 5 — Root cause hypotheses (in-chat only)

Based on trace results, compose ≥2 hypotheses **in chat** — do NOT write a separate file. They persist as in-memory state until Step 7 folds them into FIX-PLAN.md.

Render format (print to chat, one block per hypothesis):

```
🧩 Hypothesis 1 — <short label>
   Evidence:
     • <file>:<line> — <quote or behavior>
     • Recent change: <commit SHA + date> (if applicable)
     • <other signal>
   Confidence: High / Medium / Low
   Fix scope estimate: <N files, 1 layer>

🧩 Hypothesis 2 — <short label>
   Evidence: ...
   Confidence: ...
   Fix scope estimate: ...
```

Rules:
- **Minimum 2 hypotheses** — even if one looks obvious. A weak alternative (Low confidence) prevents confirmation bias.
- **Affected code paths** belong inside each hypothesis's Evidence bullets — no separate "Affected Code Paths" list.
- If Step 4 trace returned zero evidence, at least one hypothesis MUST say `Evidence: none — trace returned no matches` rather than invent paths.

Store hypotheses into shell vars `$HYP_1_LABEL`, `$HYP_1_EVIDENCE`, `$HYP_2_LABEL`, `$HYP_2_EVIDENCE`, etc. — they'll be referenced by Step 6 menu and Step 7 FIX-PLAN composition.

---

## Step 6 — Root cause confirmation

```json
{"questions": [{"question": "Most likely root cause?", "header": "Hypothesis", "multiSelect": false, "options": [
  {"label": "<Hypothesis 1 label>", "description": "<1-line evidence summary>"},
  {"label": "<Hypothesis 2 label>", "description": "<1-line evidence summary>"},
  {"label": "Other — I have a different theory", "description": "Describe in Other"}
]}]}
```

Store the picked hypothesis into shell vars:
- `$ROOT_CAUSE_LABEL` — the chosen hypothesis's label
- `$ROOT_CAUSE_EVIDENCE` — its full evidence block (file:line bullets, commit refs, etc.)
- `$REJECTED_HYPOTHESES` — array/newline-separated: for each NOT-chosen hypothesis, `<label> — <why-rejected-1-line>` (why-rejected is inferred from evidence strength + dev's pick; OK to say "Lower confidence" if no richer signal).

These feed Step 7 FIX-PLAN composition. **No file is written in Step 6** — the FIX-PLAN in Step 7 captures everything needed for audit trail (chosen + rejected).

"Other" branch: prompt dev for their theory, store as `$ROOT_CAUSE_LABEL` + `$ROOT_CAUSE_EVIDENCE = <dev's theory text>`.

### Session init (after root cause confirmed)

```bash
BASE_SLUG="fix-$(slugify "$BUG_TITLE")"
SLUG="$BASE_SLUG"
N=2

# Collision check — never reuse an existing session dir or branch silently
while [ -d "$PROJECT_ROOT/.compass/.state/sessions/$SLUG" ] \
   || git -C "$PROJECT_ROOT" show-ref --verify --quiet "refs/heads/fix/$SLUG"; do
  SLUG="${BASE_SLUG}-${N}"
  N=$((N + 1))
  [ "$N" -gt 20 ] && { echo "✗ Too many slug collisions for '$BASE_SLUG'. Pick a different bug title."; exit 1; }
done

SESSION_DIR="$PROJECT_ROOT/.compass/.state/sessions/$SLUG"
mkdir -p "$SESSION_DIR"

# Build state JSON safely via jq (handles quotes, backslashes, newlines in $BUG_TITLE etc.)
STATE_JSON=$(jq -n \
  --arg id   "$SLUG" \
  --arg typ  "dev" \
  --arg tsk  "fix" \
  --arg cat  "code" \
  --arg stk  "${STACK:-}" \
  --arg st   "fix" \
  --arg at   "$(date -u +%FT%TZ)" \
  '{session_id:$id, type:$typ, task_type:$tsk, category:$cat, is_hotfix:true,
    stack:$stk, status:$st, created_at:$at}')

compass-cli state update "$SESSION_DIR" "$STATE_JSON"
```

Apply `core/shared/git-context.md` Parts B + C with `IS_HOTFIX=true` — ALWAYS confirms before creating the `fix/<slug>` branch, even on clean base (see git-context.md Part B hotfix rule).

---

## Step 7 — Compose FIX-PLAN.md (single consolidated artifact)

Write ONE file that replaces the old trio of RESEARCH + CONTEXT + FIX-PLAN. Keep it tight — no generic constraint bullets (sub-agent HARD LIMITS in Step 10 already enforce "no refactor / no API change / preserve tests"), no nested 3-level patch structure.

### Template

```markdown
# Fix — <bug title>

**Symptom**: <1 line from input — error message, stack trace summary, described behavior>

## Root cause
<1-2 sentences of chosen hypothesis + concrete evidence: `<file>:<line>` + commit ref if applicable>

**Rejected alternatives** *(audit)*:
- <alt label> — <why rejected: lower confidence / contradicting evidence / out of scope>
- <alt label> — <why rejected>

## Patch
- File: `<path>` @ L<N> — <concrete change, 1 line, e.g. "Add null check on user.id before .toUpperCase()">
- File: `<path>` @ L<N>-<M> — <concrete change>

## Verify
- `<command 1>` *(expected pass)*
- `<command 2>`
- `<manual: browser step or curl>` *(manual)*
```

### Rules

- **Symptom** is inline (`**Symptom**:`), not a section heading — saves a line for 1-line content.
- **Root cause** gets 1-2 sentences, not a full essay. Evidence embedded as `file:line` inline refs.
- **Rejected alternatives** is a short audit trail — 1 line per rejected hypothesis. This replaces the old separate RESEARCH.md file.
- **Patch** is flat: one bullet per file. The `File: \`<path>\`` marker stays for Step 8 regex extraction. `@ L<N>` gives line info; `@ L<N>-<M>` for ranges.
- **No "Constraints" section** — sub-agent prompt HARD LIMITS enforce those universally; writing them per-fix is noise.
- **Verify** is a flat bullet list of commands. Use `*(manual)*` suffix only for non-scriptable steps.

### Write

```bash
cat > "$SESSION_DIR/FIX-PLAN.md" <<MARKDOWN
# Fix — $BUG_TITLE

**Symptom**: $BUG_DESC_ONE_LINE

## Root cause
$ROOT_CAUSE_ONE_LINE

$(if [ -n "$ROOT_CAUSE_EVIDENCE" ]; then
  echo ""
  echo "Evidence:"
  echo "$ROOT_CAUSE_EVIDENCE" | sed 's/^/- /'
fi)

**Rejected alternatives** *(audit)*:
$(echo "$REJECTED_HYPOTHESES" | sed 's/^/- /')

## Patch
$(echo "$PATCH_BULLETS")

## Verify
$(echo "$VERIFY_BULLETS")
MARKDOWN
```

`$PATCH_BULLETS` and `$VERIFY_BULLETS` are composed by the LLM from the chosen root cause + dev's domain knowledge. Each patch bullet MUST start with `- File: \`<path>\`` so Step 8 regex can parse reliably.

---

## Step 8 — Scope guard

Extract, dedupe, and validate the file list from FIX-PLAN.md before anything else:

```bash
# Extract + dedupe + drop empties
AFFECTED_LIST=$(grep -oE 'File: `[^`]+`' "$SESSION_DIR/FIX-PLAN.md" \
  | sed 's/^File: `//; s/`$//' \
  | awk 'NF' | sort -u)

AFFECTED_FILES=$(echo "$AFFECTED_LIST" | awk 'NF' | wc -l | tr -d ' ')

# Layer detection — broader, covers JS/TS, Go, Rust, Python, monorepo, mobile
AFFECTED_LAYERS=$(echo "$AFFECTED_LIST" | awk -F'/' '{
  for (i=1; i<=NF; i++) {
    if ($i ~ /^(src|app|apps|packages|internal|pkg|cmd|lib|ios|android|macos|web|api|server|client|frontend|backend|services|handlers|components|pages|config|infra)$/) {
      print $i; next
    }
  }
  if (NF > 0) print $1
}' | sort -u | wc -l | tr -d ' ')

echo "  AFFECTED_FILES=$AFFECTED_FILES  AFFECTED_LAYERS=$AFFECTED_LAYERS"
```

### Hard cap (non-negotiable)

```bash
if [ "$AFFECTED_FILES" -eq 0 ]; then
  echo "✗ FIX-PLAN has no 'File: \`...\`' entries. Cannot dispatch."
  echo "  Fix FIX-PLAN.md manually or re-run /compass:fix."
  # Stop the workflow here. Do NOT dispatch Step 10.
  exit 0
fi

if [ "$AFFECTED_FILES" -gt 20 ]; then
  echo "✗ Hard cap: hotfix scope is limited to 20 files; FIX-PLAN lists $AFFECTED_FILES."
  echo "  This is never a hotfix. Run:"
  echo "    /compass:spec \"$BUG_DESC\""
  echo "    /compass:prepare"
  echo "    /compass:cook"
  # Do NOT offer an override. Stop workflow.
  exit 0
fi
```

### Soft check (6–20 files OR >1 layer)

```bash
if [ "$AFFECTED_FILES" -gt 5 ] || [ "$AFFECTED_LAYERS" -gt 1 ]; then
  # AskUserQuestion — with the numbers substituted
  :
fi
```

en:
```json
{"questions": [{"question": "Scope beyond typical hotfix ($AFFECTED_FILES files, $AFFECTED_LAYERS layer(s)). Proceed?", "header": "Scope check", "multiSelect": false, "options": [
  {"label": "Continue hotfix flow", "description": "Accept scope; single-wave fix. Risky above 10 files — main agent will re-verify each file."},
  {"label": "Switch to full spec flow (Recommended)", "description": "Abort — use /compass:spec + /compass:prepare + /compass:cook. Safer for multi-file / multi-layer changes."},
  {"label": "Cancel", "description": "Stop, rethink scope manually"}
]}]}
```

vi: translate (`Tiếp tục hotfix`, `Chuyển sang spec flow (Khuyến nghị)`, `Huỷ`).

On "Switch" → print: `ℹ Scope > hotfix. Run: /compass:spec "$BUG_DESC"` and stop.
On "Cancel" → stop.
On "Continue" → proceed to Step 9. Persist `AFFECTED_LIST` into **state.json** (NOT a separate `.files-affected` file) so Step 10 can read it via jq:

```bash
# Build JSON array of files from newline-separated list, merge into state.json
FILES_JSON=$(echo "$AFFECTED_LIST" | awk 'NF' | jq -R . | jq -s .)
compass-cli state update "$SESSION_DIR" "$(jq -n \
  --argjson files "$FILES_JSON" \
  '{files_affected: $files}')"
```

---

## Step 9 — Dev review + approve

### 9a. MANDATORY render — print the key blocks into chat

Do not just tell the dev "FIX-PLAN has been written". Extract and display each block verbatim so the dev can decide without opening the file. Format:

```
📋 Fix — <BUG_TITLE>
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🎯 Root cause
<Verbatim contents of the "## Root cause" section from FIX-PLAN.md>

📝 Patch (<N> file(s), <L> layer(s))
<Verbatim contents of the "## Patch" section>

🧪 Verify (<V> command(s))
<Verbatim contents of the "## Verify" section>

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Full plan: $SESSION_DIR/FIX-PLAN.md
```

Extraction helper — parse sections by heading. Note: Symptom is inline (`**Symptom**: ...`), not a `## Symptom` heading, so it's not extracted as a block; it's surfaced inside Root cause context if relevant.

```bash
# Extract block between "## <heading>" and the next "## " (or EOF)
extract_section() {
  local heading="$1"; local file="$2"
  awk -v h="## $heading" '
    $0 == h { flag=1; next }
    /^## / && flag { exit }
    flag { print }
  ' "$file"
}

ROOT_CAUSE_BLOCK=$(extract_section "Root cause" "$SESSION_DIR/FIX-PLAN.md")
PATCH_BLOCK=$(extract_section "Patch" "$SESSION_DIR/FIX-PLAN.md")
VERIFY_BLOCK=$(extract_section "Verify" "$SESSION_DIR/FIX-PLAN.md")
```

If any block is empty → do NOT proceed. Print `✗ FIX-PLAN missing "## <heading>" section — fix it before approve.` and stop.

### 9b. Approve

```json
{"questions": [{"question": "Fix plan OK?", "header": "Approve", "multiSelect": false, "options": [
  {"label": "OK, implement", "description": "Spawn sub-agent, apply patch, run verify commands"},
  {"label": "Show full FIX-PLAN", "description": "Print the entire FIX-PLAN.md including Symptom + Rejected alternatives, then re-ask"},
  {"label": "Adjust plan", "description": "I'll edit FIX-PLAN.md manually. When done, reply 'done' and I'll re-read + re-render Step 9a"},
  {"label": "Wrong root cause — re-trace", "description": "Back to Step 4 with new hypothesis"},
  {"label": "Cancel", "description": "Abort — session dir kept for debug"}
]}]}
```

vi: translate (`Triển khai`, `Xem FIX-PLAN đầy đủ`, `Tôi sẽ sửa plan`, `Sai root cause — trace lại`, `Huỷ`).

- **"OK, implement"** → proceed to Step 10.
- **"Show full FIX-PLAN"** → `cat "$SESSION_DIR/FIX-PLAN.md"` verbatim into chat, then re-render Step 9b (not 9a — the 3 blocks already shown).
- **"Adjust plan"** → pause, wait for dev to signal done, then re-extract blocks + re-render Step 9a (full loop).
- **"Wrong root cause"** → clear `$ROOT_CAUSE_LABEL` + `$ROOT_CAUSE_EVIDENCE` + any draft FIX-PLAN.md, loop back to Step 4.
- **"Cancel"** → stop.

---

## Step 10 — Execute (single wave)

Apply `core/shared/wave-execution.md` for a single-wave "fix" variant.

### 10a. Prepare worker rules

```bash
# Load worker rules (same pattern as cook)
if [ -f "$PROJECT_ROOT/.compass/worker-rules.md" ]; then
  WORKER_RULES=$(cat "$PROJECT_ROOT/.compass/worker-rules.md")
else
  WORKER_RULES=$(cat "$HOME/.compass/core/worker-rules/base.md")
fi

TECH_STACK=$(echo "$CONFIG" | jq -r '.tech_stack // [] | .[]' 2>/dev/null)
for STACK in $TECH_STACK; do
  ADDON="$HOME/.compass/core/worker-rules/addons/$STACK.md"
  if [ -f "$ADDON" ]; then
    # Use printf so \n becomes a real newline (bash "\n" in double quotes is literal)
    WORKER_RULES=$(printf '%s\n---\n%s' "$WORKER_RULES" "$(sed '1,/^---$/d' "$ADDON")")
  fi
done
```

### 10b. Reuse the validated file list from Step 8 (via state.json)

```bash
# Read files_affected[] persisted by Step 8 — do NOT re-extract from FIX-PLAN.
# This avoids volatility if the LLM reformats FIX-PLAN between steps.
FILES_AFFECTED=$(jq -r '.files_affected // [] | .[]' "$SESSION_DIR/state.json" 2>/dev/null)

if [ -z "$FILES_AFFECTED" ]; then
  echo "✗ state.json has no files_affected[] — Step 8 must run first."
  exit 1
fi

FILES_COUNT=$(echo "$FILES_AFFECTED" | awk 'NF' | wc -l | tr -d ' ')
if [ "$FILES_COUNT" -eq 0 ] || [ "$FILES_COUNT" -gt 20 ]; then
  echo "✗ Invariant broken: files_affected count = $FILES_COUNT. Aborting dispatch."
  exit 1
fi

# Space-joined form for the sub-agent prompt
FILES_AFFECTED_JOINED=$(echo "$FILES_AFFECTED" | tr '\n' ' ' | sed 's/ $//')
```

### 10c. Build sub-agent prompt

```bash
PROMPT=$(cat <<END
# Hotfix — $BUG_TITLE

You are applying a single-file (or small multi-file) hotfix. Read FIX-PLAN below, make the change, verify with the listed test command, report back.

## GitNexus
GitNexus: $GITNEXUS_STATUS
GitNexus Repo: $GITNEXUS_REPO
If GitNexus is GITNEXUS_AVAILABLE, run gitnexus_impact({target: "symbolName", direction: "upstream", repo: "$GITNEXUS_REPO"}) before modifying any symbol. If risk is HIGH or CRITICAL, report back instead of proceeding.

## Strict scope rules (HARD LIMITS)
- Files you may modify: $FILES_AFFECTED_JOINED
- Do NOT modify any file outside this whitelist
- Do NOT create new files unless a listed entry does not yet exist (creating is ONLY allowed to satisfy a whitelist entry)
- Do NOT refactor unrelated code
- Do NOT run git commit, git add, git push, or any git state-changing command
- Do NOT run npm install, yarn install, pip install, or any dependency-changing command
- If the whitelist looks wrong or insufficient to fix the bug, STOP and report back with status "needs_human"

## FIX-PLAN (single source of truth — contains symptom, root cause, rejected alternatives, patch, verify)
$(cat "$SESSION_DIR/FIX-PLAN.md")

## Worker Rules
$WORKER_RULES

## Execution steps
1. Read each file in the whitelist (only the ones that exist)
2. Apply the patch per FIX-PLAN — minimal edits, no incidental cleanup
3. Run each verification command from FIX-PLAN
4. If a test fails: read output, make a targeted fix, re-run (max 2 retries)
5. Report back with EXACTLY this JSON shape:
{
  "status": "success" | "needs_human" | "partial",
  "files_changed": [{"path": "...", "change_summary": "..."}],
  "tests_run": [{"command": "...", "exit_code": N, "output_excerpt": "..."}],
  "retries_used": 0,
  "notes": "..."
}
END
)
```

### 10d. Dispatch — MANDATORY Agent tool call

**This is not a bash command.** Stop writing bash. The orchestrator MUST now invoke the `Agent` tool exactly once:

```
Agent(
  description: "Apply hotfix — <bug title, ≤40 chars>",
  subagent_type: "general-purpose",
  prompt: <contents of $PROMPT built in 10c>
)
```

Rules:
- Do NOT apply the fix inline in the orchestrator context. The sub-agent must run in a fresh context window.
- Do NOT spawn multiple sub-agents. Exactly one call for the single wave.
- Wait for the sub-agent to return before moving to 10e.

### 10e. Parse sub-agent response

```bash
# Capture the sub-agent's reported status from its return message.
# Store in $WORKER_STATUS (one of: success | needs_human | partial)
# Store the parsed JSON in $SESSION_DIR/.worker-report.json for audit.
```

Branch:
- `success` → proceed to 10f (main-agent re-verify).
- `needs_human` → AskUserQuestion below.
- `partial` → AskUserQuestion below.

en:
```json
{"questions": [{"question": "Sub-agent reported '$WORKER_STATUS'. What now?", "header": "Worker result", "multiSelect": false, "options": [
  {"label": "Retry with guidance", "description": "Type a hint in Other — I'll re-spawn the sub-agent with your guidance appended"},
  {"label": "Abort fix", "description": "Stop workflow. Keep session dir for inspection. No commit."},
  {"label": "Escalate to full spec flow", "description": "This bug needs more than a hotfix — run /compass:spec"}
]}]}
```

vi: translate (`Retry với gợi ý`, `Huỷ fix`, `Chuyển sang spec flow`).

Retry max 2 additional times. After 3 total attempts, force `Abort`.

### 10f. Main-agent re-verify (MANDATORY — preview + run)

**Extract verification commands from FIX-PLAN.md:**

```bash
VERIFY_CMDS=$(awk '/^## Verify/{flag=1; next} /^## /{flag=0} flag' "$SESSION_DIR/FIX-PLAN.md" \
  | grep -oE '`[^`]+`' | sed 's/`//g')

VERIFY_COUNT=$(echo "$VERIFY_CMDS" | awk 'NF' | wc -l | tr -d ' ')
```

#### 10f-preview — show commands + ask before running

If `VERIFY_COUNT = 0`:
```
ℹ No verification commands parsed from FIX-PLAN (none found in `backticks` under "## Verify").
  Skipping re-verify. You'll need to test manually after fix.
```
Set `VERIFY_FAILED=0` and proceed to Step 11.

If `VERIFY_COUNT > 0`, render:
```
🧪 About to run $VERIFY_COUNT verify command(s):
  1. <cmd1>
  2. <cmd2>
  ...
Working directory: $PROJECT_ROOT
Per-command timeout: 300s
```

Flag potentially destructive commands — any of: `rm `, `rm -`, `> `, `>>`, `dd `, `mkfs`, `deploy`, `publish`, `push `, `curl `, `wget `, `sudo `, `:(){`, `git push`, `npm publish`, `yarn publish`, `chmod -R`, `chown -R`. If any match, add:
```
⚠ One or more commands look destructive or network-facing. Review carefully.
```

en:
```json
{"questions": [{"question": "Run these $VERIFY_COUNT verify command(s)?", "header": "Verify", "multiSelect": false, "options": [
  {"label": "Run all (Recommended)", "description": "Execute each command in $PROJECT_ROOT with 300s timeout"},
  {"label": "Run one-by-one", "description": "Confirm each command individually before running"},
  {"label": "Skip verify", "description": "⚠ Fix is unverified — commit gate will still block commit unless you force"},
  {"label": "Edit verify commands", "description": "Pause — edit '## Verify' in FIX-PLAN.md, then reply 'done'"}
]}]}
```

vi: translate (`Chạy hết (Khuyến nghị)`, `Chạy từng lệnh`, `Bỏ qua verify`, `Sửa verify commands`).

#### 10f-run — execute based on choice

```bash
VERIFY_FAILED=0

run_one() {
  local cmd="$1"
  echo "▶ Verifying: $cmd"
  if ! timeout 300s bash -c "cd \"$PROJECT_ROOT\" && $cmd"; then
    echo "  ✗ Failed (exit=$?): $cmd"
    VERIFY_FAILED=1
    return 1
  fi
  echo "  ✓ Passed"
  return 0
}

case "$VERIFY_CHOICE" in
  run_all)
    while IFS= read -r cmd; do
      [ -z "$cmd" ] && continue
      run_one "$cmd"
    done <<< "$VERIFY_CMDS"
    ;;
  one_by_one)
    # For each cmd, AskUserQuestion: Run / Skip this one / Abort
    # Run via run_one; Skip marks that cmd skipped (not failed); Abort stops verify and sets VERIFY_FAILED=1
    :
    ;;
  skip)
    VERIFY_FAILED=-1   # special: user skipped, not passed
    ;;
  edit)
    # Pause; on resume, re-run Step 10f from preview
    :
    ;;
esac

echo "VERIFY_FAILED=$VERIFY_FAILED"
```

Outcomes:
- `VERIFY_FAILED=0` → continue to Step 11.
- `VERIFY_FAILED=1` → treat same as `needs_human` in 10e (do NOT commit).
- `VERIFY_FAILED=-1` (skipped) → continue, but Step 11a will show a warning and require explicit "Commit anyway" confirmation.

---

## Step 11 — Chain: test → commit

### 11a. Gate — only proceed if Step 10 succeeded

```bash
# Hard gate — sub-agent must have reported success.
if [ "$WORKER_STATUS" != "success" ]; then
  echo "ℹ Sub-agent did not report success (worker=$WORKER_STATUS). Not committing."
  echo "  Changes left unstaged for manual review:"
  echo "$FILES_AFFECTED" | tr ' ' '\n' | awk 'NF'
  compass-cli state update "$SESSION_DIR" '{"status":"blocked","reason":"worker_not_success"}' 2>/dev/null || true
  exit 0
fi

# Hard gate — verify failures block commit.
if [ "${VERIFY_FAILED:-0}" -eq 1 ]; then
  echo "✗ Verify commands failed. Not committing."
  compass-cli state update "$SESSION_DIR" '{"status":"blocked","reason":"verify_failed"}' 2>/dev/null || true
  exit 0
fi
```

**Soft gate — verify was skipped:**

If `VERIFY_FAILED=-1` (user chose "Skip verify" at Step 10f), ask explicit confirmation before committing an unverified fix:

en:
```json
{"questions": [{"question": "⚠ Fix was NOT verified. Commit anyway?", "header": "Unverified commit", "multiSelect": false, "options": [
  {"label": "Commit anyway", "description": "I accept the risk — no tests were run to confirm the fix works"},
  {"label": "Run verify now", "description": "Go back to Step 10f and run the verify commands"},
  {"label": "Cancel commit", "description": "Stop — keep changes unstaged for manual review"}
]}]}
```

vi: translate (`Commit luôn`, `Chạy verify ngay`, `Huỷ commit`).

- "Commit anyway" → proceed to 11b (tests chain still available).
- "Run verify now" → jump back to Step 10f preview.
- "Cancel commit" → stop.

### 11b. Chain to compass:test (optional)

en:
```json
{"questions": [{"question": "Run tests before committing?", "header": "Test", "multiSelect": false, "options": [
  {"label": "Run compass:test (Recommended)", "description": "Verify fix didn't break anything else"},
  {"label": "Skip tests", "description": "Commit without testing"}
]}]}
```

vi: translate (`Chạy compass:test (Khuyến nghị)` / `Bỏ qua tests`).

If "Run compass:test" → invoke `/compass:test` workflow inline (read and execute `~/.compass/core/workflows/test.md`). Wait for results. If test.md reports failures → re-gate 11a semantics (do not auto-commit).

### 11c. Stage files (safely quoted)

```bash
# NEVER use unquoted $FILES_AFFECTED with git add — filenames with spaces break.
STAGE_ERR=0
while IFS= read -r f; do
  [ -z "$f" ] && continue
  if [ ! -e "$PROJECT_ROOT/$f" ] && [ ! -e "$f" ]; then
    echo "  ⚠ Skipping (not found): $f"
    continue
  fi
  git -C "$PROJECT_ROOT" add -- "$f" || STAGE_ERR=1
done <<< "$(echo "$FILES_AFFECTED" | tr ' ' '\n' | awk 'NF')"

if [ "$STAGE_ERR" -ne 0 ]; then
  echo "✗ git add failed for one or more files. Aborting commit."
  exit 1
fi

# Verify something is actually staged
if git -C "$PROJECT_ROOT" diff --cached --quiet; then
  echo "ℹ Nothing staged. Sub-agent may not have modified any file. Skipping commit."
  exit 0
fi
```

### 11d. Derive commit scope (broad layout support)

```bash
# Handle src/, apps/, packages/, internal/, pkg/, cmd/, ios/, android/, macos/, etc.
SCOPE=$(echo "$FILES_AFFECTED" | tr ' ' '\n' | awk -F'/' 'NF {
  # Strip common layout prefixes, return the next segment if present
  layout="^(src|app|apps|packages|internal|pkg|cmd|lib)$"
  if ($1 ~ layout && NF >= 2) { print $2; next }
  print $1
}' | sort -u | head -1)

# Safe fallback
[ -z "$SCOPE" ] && SCOPE="core"
# Strip file extensions in case we picked up a top-level file name
SCOPE=$(echo "$SCOPE" | sed 's/\.[^.]*$//')

SUMMARY=$(printf '%s' "$BUG_TITLE" | head -c 72)
MSG="fix($SCOPE): $SUMMARY"
```

### 11e. Commit — preview diff first

**Show staged changes before asking.** Do not ask "Commit?" in the abstract.

```bash
# Print summary (stat) + a short sample of the diff
echo "📊 Staged changes"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
git -C "$PROJECT_ROOT" diff --cached --stat
echo ""
echo "📝 Diff preview (first 200 lines)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
git -C "$PROJECT_ROOT" diff --cached | head -200
DIFF_LINES=$(git -C "$PROJECT_ROOT" diff --cached | wc -l | tr -d ' ')
if [ "$DIFF_LINES" -gt 200 ]; then
  echo ""
  echo "… ($DIFF_LINES lines total — $((DIFF_LINES - 200)) lines truncated. Pick 'Show full diff' to see all.)"
fi
echo ""
echo "💬 Proposed commit message"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "$MSG"
echo ""
```

Render the AskUserQuestion AFTER the preview is printed.

en:
```json
{"questions": [{"question": "Commit this fix?", "header": "Commit", "multiSelect": false, "options": [
  {"label": "Commit (Recommended)", "description": "Use the generated message shown above"},
  {"label": "Show full diff", "description": "Print the entire git diff --cached, then re-ask"},
  {"label": "Edit message", "description": "Type your own commit message"},
  {"label": "Unstage and cancel", "description": "git reset HEAD — leave changes in working tree for manual review"},
  {"label": "Cancel (keep staged)", "description": "Stop — changes stay staged, no commit"}
]}]}
```

vi: translate (`Commit (Khuyến nghị)`, `Xem diff đầy đủ`, `Sửa commit message`, `Unstage + huỷ`, `Huỷ (giữ staged)`).

- **"Commit"** → `compass-cli git commit "$MSG" || git -C "$PROJECT_ROOT" commit -m "$MSG"`.
- **"Show full diff"** → `git -C "$PROJECT_ROOT" diff --cached` (verbatim), then re-render the AskUserQuestion (not the preview).
- **"Edit message"** → prompt for custom message via AskUserQuestion Other field, commit with that.
- **"Unstage and cancel"** → `git -C "$PROJECT_ROOT" reset HEAD -- .` (scoped to staged files only), then stop.
- **"Cancel (keep staged)"** → stop; dev deals manually.

```bash
COMMIT_SHA=$(git -C "$PROJECT_ROOT" rev-parse HEAD)
compass-cli state update "$SESSION_DIR" "$(jq -n \
  --arg status "complete" \
  --arg sha "$COMMIT_SHA" \
  --arg at "$(date -u +%FT%TZ)" \
  '{status:$status, commit_sha:$sha, completed_at:$at}')"
```

---

## Step 12 — Hand-off

Print (adapted to `$LANG`):

- en:
```
✓ Fix applied.
  Session:  <slug>
  Commit:   <sha>
  Tests:    passed / skipped
  Branch:   fix/<slug>

  Ship when ready:
    git push -u origin fix/<slug>
    gh pr create --title "fix: <summary>"
```

- vi: same content, translated.

Stop. Do NOT auto-push.

---

## Edge cases

| Situation | Handling |
|---|---|
| No PR history to trace from | Step 4 falls back to keyword grep only — may miss recent regressions |
| Dev rejects all hypotheses | Step 6 "Other" branch — dev provides theory, re-compose FIX-PLAN |
| FIX-PLAN exceeds scope limit | Step 8 offers redirect to full spec flow |
| Sub-agent can't apply patch (file locked, merge conflict) | Treat as "needs_human", prompt dev |
| Test command fails but sub-agent reports success | Step 10 main-agent re-verify catches it |
| Failing test is a pre-existing issue unrelated to bug | Dev decides in scope-guard: include fix or skip |
| Bug is a false report (code is correct) | Dev picks "Cancel" at Step 9 — session kept with hypothesis notes |
| Fix touches config + code | Step 8 flags as >1 layer; dev decides |
| Retry loop fails 3 times | AskUserQuestion (retry-with-guidance / open-as-full-spec / abort) |
| Cross-layer trace takes >60s | Partial trace used; note in FIX-PLAN Root cause section ("Trace partial — <reason>") |
| Commit hook rejects (linting, pre-commit) | Print error, pause; dev resolves hook issue then resumes `/compass:fix` |

---

## Final — Hand-off

Step 12 handled it. Stop cleanly.
