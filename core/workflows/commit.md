# Workflow: compass:commit

Smart commit — auto-generate a conventional commit message from staged or changed files.

**Purpose**: Inspect working tree, help PO stage files if needed, generate a conventional commit message from the diff, confirm, and commit.

**Input**: Git working directory inside `$PROJECT_ROOT`
**Output**: A single conventional commit

---

## Step 0 — Resolve active project

Apply the shared snippet from `core/shared/resolve-project.md`. It sets up `$PROJECT_ROOT`, `$CONFIG`, and `$PROJECT_NAME` for downstream steps and prints the "Using: <name>" banner.

From `$CONFIG`, extract: `lang`. If missing → tell user to run `/compass:init` first and stop.

All user-facing text from this point uses `$LANG` (vi or en).

---

## Step 1 — Check git status

Run inside `$PROJECT_ROOT`:

```bash
cd "$PROJECT_ROOT"
STAGED=$(git diff --cached --name-only)
CHANGED=$(git diff --name-only)
UNTRACKED=$(git ls-files --others --exclude-standard)
```

---

## Step 2 — Smart staging

- **If `$STAGED` is non-empty** → files are already staged, skip to Step 3.
- **If `$STAGED` is empty but `$CHANGED` or `$UNTRACKED` is non-empty** → AskUserQuestion (AI translates per `$LANG` — see ux-rules Language Policy):
  - Question: `"No files staged yet. What would you like to do?"`
  - Options:
    - **"Stage all changed files"** → run `git add -A`
    - **"Pick files to stage"** → show combined list of `$CHANGED` + `$UNTRACKED` as a multiSelect AskUserQuestion. Stage only selected files via `git add <file>...`
    - **"Cancel"** → stop workflow, print cancellation message.
- **If all three are empty** → print `"Nothing to commit."` and stop.

---

## Step 3 — Read diff for message generation

```bash
cd "$PROJECT_ROOT"
git diff --cached --stat
git diff --cached
```

Read the output carefully — it is the basis for the commit message.

---

## Step 4 — Generate conventional commit message

Build a message in the format `<type>(<scope>): <description>`:

1. **Detect type** from the diff:
   - New files (create mode) → `feat`
   - Modified existing files → `fix` or `refactor` (use judgment: bug-related → `fix`, structural → `refactor`)
   - Config / CI / build files (e.g. `package.json`, `.yml`, `Makefile`) → `chore`
   - Documentation files (`.md`, `docs/`) → `docs`
   - Test files → `test`
   - Mixed → pick the dominant type

2. **Detect scope** from file paths:
   - Find the deepest common directory prefix across all changed files.
   - Use the most meaningful segment (e.g. `cli`, `core`, `api`, `ui`).
   - If files span the entire repo, omit scope: `<type>: <description>`

3. **Description**: one-line summary of what changed (imperative mood, lowercase, no period).

---

## Step 5 — Confirm and commit

Show the generated message and AskUserQuestion (AI translates per `$LANG` — see ux-rules Language Policy):
- Question: `"Message: <message>. What would you like to do?"`
- Options:
  - **"Commit"** → run `git commit -m "<message>"`
  - **"Edit message"** → AskUserQuestion with a free-text input asking for the custom message, then `git commit -m "<custom_message>"`
  - **"Cancel"** → run `git reset HEAD` to unstage, print cancellation message, stop.

---

## Step 6 — Print result

After a successful commit, print:
- The commit SHA (from `git rev-parse --short HEAD`)
- The number of files committed
- The commit message used

Print: `"Committed <SHA> — <N> file(s): <message>"` (AI translates per `$LANG` — see ux-rules Language Policy).
