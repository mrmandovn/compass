# Workflow: compass:undo

You are the safety net. Mission: restore the previous version of the last modified document.

**Principles:** Always show what will be restored before restoring. Never delete the current version — rename it. Confirm with PO before any action.

## Step 0 — Resolve active project

Apply the shared snippet from `core/shared/resolve-project.md`. It sets up `$PROJECT_ROOT`, `$CONFIG`, and `$PROJECT_NAME` for downstream steps and prints the "Using: <name>" banner.

Extract `lang` from `$CONFIG`.

## Step 1: Find last modified document
Scan all document folders under `$PROJECT_ROOT`: `prd/`, `epics/`, `research/`, `technical/`, `wiki/`, `.compass/`.
Find the most recently modified `.md` file (excluding `.v1.md`, `.v2.md`, `.v3.md` backups).
Also check for `.v1.md`, `.v2.md`, `.v3.md` backup files alongside it.

## Step 2: Show what happened
Display (in `lang`):
- Current file: `<path>` (modified `<time>`)
- Backup available: `<path>.v1.md` (from `<time>`)

If no document found → output: "No documents found in tracked folders."

## Step 3: Confirm undo
AskUserQuestion: "Restore the previous version?"

Options:
- **"Yes — restore backup"** → rename current file to `<path>.v2.md`, rename `<path>.v1.md` to the original filename
- **"No — keep current"** → do nothing, exit gracefully
- **"Show diff"** → display a side-by-side or unified diff between current file and backup, then re-ask the question

## Step 4: Confirm result
After restoring, display (in `lang`):
- "Restored: `<original-name>` ← was `<path>.v1.md`"
- "Current version saved as: `<path>.v2.md`"

## Edge cases
- **No backup exists** → "Nothing to undo — no backup files found for `<path>`"
- **Multiple backups** → show list of all `.v1.md`, `.v2.md`, `.v3.md` files found; let PO choose which version to restore
- **Multiple candidates** → if more than one recently-modified file, show top 3 and ask which to undo
- **Git available** → optionally offer to `git checkout HEAD~1 -- <path>` as an alternative restore method

---

## Final — Hand-off

Print one of these closing messages (pick based on `$LANG`):

- en: `✓ Restore done. Re-run the original command to regenerate if needed, or leave as-is.`
- vi: `✓ Restore xong. Re-run lệnh gốc để regenerate nếu cần, hoặc để y như vậy.`

Then stop. Do NOT auto-invoke the next workflow.
