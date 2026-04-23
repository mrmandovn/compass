# Workflow: compass:test

You are the test runner. Mission: execute acceptance tests from a TEST-SPEC.md or detect and run the project's test suite.

**Principles:** Run every test. Report per-requirement results. Never skip failures — surface them clearly with stderr context.

**Purpose**: Run tests — either acceptance criteria extracted from the latest TEST-SPEC.md session, or the project's auto-detected test suite.

**Input**: Active project (resolved via Step 0)
**Output**: Per-test pass/fail report with summary

---

Apply the UX rules from `core/shared/ux-rules.md`.

---

## Step 0 — Resolve active project

Apply the shared snippet from `core/shared/resolve-project.md`. It sets up `$PROJECT_ROOT`, `$CONFIG`, and `$PROJECT_NAME` for downstream steps and prints the "Using: <name>" banner.

From `$CONFIG`, extract: `lang`. If missing → tell user to run `/compass:init` first and stop.

All output from this point is in `lang`.

---

## Step 1 — Find latest TEST-SPEC.md

```bash
SESSIONS_DIR="$PROJECT_ROOT/.compass/.state/sessions"
if [ -d "$SESSIONS_DIR" ]; then
  LATEST_SESSION=$(find "$SESSIONS_DIR" -name "TEST-SPEC.md" -exec stat -f "%m %N" {} \; 2>/dev/null | sort -rn | head -1 | awk '{print $2}')
  # Fallback if macOS stat fails
  if [ -z "$LATEST_SESSION" ]; then
    LATEST_SESSION=$(find "$SESSIONS_DIR" -name "TEST-SPEC.md" -print0 | xargs -0 ls -t 2>/dev/null | head -1)
  fi
fi
```

If `$LATEST_SESSION` is non-empty → proceed to **Step 2** (TEST-SPEC mode).
If empty or `$SESSIONS_DIR` doesn't exist → skip to **Step 3** (auto-detect mode).

---

## Step 2 — Run TEST-SPEC acceptance criteria

Read `$LATEST_SESSION` (the TEST-SPEC.md file).

Parse the file to extract test commands and their requirement tags:
- Extract `[REQ-xx]` tags from test block headers (e.g., `### [REQ-01] Some title`)
- Extract lines matching pattern: `**Verify**: \`<command>\`` or `- **Verify**: \`<command>\``
- Associate each Verify command with the most recent `[REQ-xx]` header above it

For each extracted command:
1. Run the command via Bash (working directory = `$PROJECT_ROOT`)
2. Capture exit code + stdout + stderr

Print results in this format:

```
Test Results (from TEST-SPEC.md)
─────────────────────────────────
REQ-01: ✅ PASS — <command excerpt>
REQ-02: ❌ FAIL — <command excerpt>
  Error: <first 3 lines of stderr>
REQ-03: ✅ PASS — <command excerpt>

Summary: 2/3 passed
```

Then proceed to **Step 4** (summary).

---

## Step 3 — Auto-detect test runner

If no TEST-SPEC.md was found, detect the project's test runner:

```bash
if [ -f "$PROJECT_ROOT/package.json" ]; then
  RUNNER="npm test"
elif [ -f "$PROJECT_ROOT/Cargo.toml" ]; then
  RUNNER="cargo test"
elif [ -f "$PROJECT_ROOT/pyproject.toml" ] || [ -f "$PROJECT_ROOT/pytest.ini" ]; then
  RUNNER="pytest -v"
elif [ -f "$PROJECT_ROOT/go.mod" ]; then
  RUNNER="go test ./... -v"
else
  echo "NO_RUNNER_DETECTED"
fi
```

**If a runner is detected** → run it via Bash with a real `timeout` wrapper so a hung test cannot block the workflow:

```bash
# 10-min cap per test suite — reasonable default for most projects.
# Override per project by setting TEST_TIMEOUT env var.
TIMEOUT_BUDGET="${TEST_TIMEOUT:-600s}"
timeout "$TIMEOUT_BUDGET" bash -c "cd \"$PROJECT_ROOT\" && $RUNNER"
EXIT_CODE=$?
if [ "$EXIT_CODE" = "124" ]; then
  echo "⚠ Test runner exceeded $TIMEOUT_BUDGET — aborted. This is a stuck-test warning, not a pass/fail signal."
fi
```

Show full output, then proceed to **Step 4**.

**If no runner is detected** → AskUserQuestion:

- vi:
```json
{"questions": [{"question": "Không tìm thấy test runner. Bạn muốn làm gì?", "header": "Test runner", "multiSelect": false, "options": [
  {"label": "Nhập lệnh test", "description": "Gõ command test của bạn"},
  {"label": "Bỏ qua", "description": "Không chạy test"}
]}]}
```

- en:
```json
{"questions": [{"question": "No test runner detected. What would you like to do?", "header": "Test runner", "multiSelect": false, "options": [
  {"label": "Type your test command", "description": "Enter a custom test command to run"},
  {"label": "Skip", "description": "Don't run any tests"}
]}]}
```

If user provides a command → run it. If skip → stop with no-op message.

---

## Step 4 — Summary

Count total tests and passed/failed. Print summary in `$LANG`:

- en: `✅ All tests passed` or `❌ X/Y tests failed`
- vi: `✅ Tất cả test đã pass` or `❌ X/Y test thất bại`

---

## Step 0c — Memory hook

On success: `compass-cli project use "$PROJECT_ROOT"` (idempotent, touches `last_used`).
