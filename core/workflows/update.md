# Workflow: compass:update

<!-- update.md does not require project resolve -->

You are the update agent. Mission: check for new versions and apply updates safely.

**Principles:** Always show what changed before updating. Never update without confirmation. Preserve local modifications.

**Purpose**: Update Compass to the latest version from GitHub.

**Output**: Status message printed to terminal. No file is created.

---

Apply the UX rules from `core/shared/ux-rules.md`. Load `lang` from `.compass/.state/config.json`.

> **Additional rule for update**: If `.compass/.state/config.json` is missing, default `lang` to `en`. The update workflow can still run without project config.

---

## Step 0 — Load config & language

Read `.compass/.state/config.json`.
- Extract `lang` (default: `en`).
- Config missing is OK — update can still run without project config.

---

## Step 1 — Check current version

Read `~/.compass/VERSION`.
- Store as `LOCAL_VERSION`.
- If the file doesn't exist, treat `LOCAL_VERSION` as `unknown`.

---

## Step 2 — Check for updates

Fetch the latest tag from GitHub:

```
curl -sf "https://api.github.com/repos/mrmandovn/compass/tags"
```

Extract the first tag name (e.g. `v0.3.0`) using:

```
python3 -c "import sys,json; tags=json.load(sys.stdin); print(tags[0]['name'] if tags else 'unknown')"
```

Store as `LATEST_VERSION`.

**Edge cases:**
- No internet / curl fails → show "Could not reach GitHub. Showing local version only." and stop after displaying local version.
- Response is empty or malformed → treat as "unknown" and warn.

---

## Step 3 — Compare versions

- If `LOCAL_VERSION == LATEST_VERSION`: print "✓ Already on latest version (v{LOCAL_VERSION})" and stop.
- If different: show a summary — current version, latest version.
- Use AskUserQuestion to confirm update:
  - Option A: "Update now" — proceed to Step 4
  - Option B: "Skip" — exit gracefully

---

## Step 4 — Update

Check if `~/.compass` is a git repo:

```
git -C ~/.compass rev-parse --is-inside-work-tree 2>/dev/null
```

**If git repo:**

```
cd ~/.compass && git pull --ff-only origin main
```

- Show success output.
- If `git pull` fails with merge conflict → warn: "There are local modifications conflicting with the update. Try: `git stash`, `git pull`, `git stash pop` to resolve."
- If local modifications detected before pull (`git status --porcelain`): warn the user before proceeding. Use AskUserQuestion to confirm.

**If not a git repo (installed via npm tarball without git):**

- Inform the user: "Compass was not installed via git. Re-run the installer to fetch the latest version:"
- Print: `npx compass-m`
- Stop here. The installer is idempotent and preserves `~/.compass/projects.json` + `~/.compass/global-config.json`.

---

## Step 5 — Re-install adapters

After pulling new code, re-copy adapters to the host so new commands are available:

Verify symlinks are intact:
```bash
ls ~/.claude/commands/compass/ 2>/dev/null | wc -l
```
If count doesn't match 21, re-run: `~/.compass/bin/install`.

---

## Step 6 — Verify

Read `~/.compass/VERSION` again.
Print: "✓ Updated to v{NEW_VERSION} (<N> commands)"

**IMPORTANT — show this after every successful update:**

- en: `"⚠ Close and reopen your AI host to load new commands. Current session only sees commands loaded at startup."`
- vi: `"⚠ Đóng và mở lại AI host để load commands mới. Session hiện tại chỉ thấy commands đã load lúc khởi động."`

---

## Edge cases summary

| Situation | Behavior |
|---|---|
| Already on latest | "✓ Already on latest version" — stop |
| No internet | Show local version, skip update |
| Not a git repo | Suggest re-running `npx compass-m` |
| Merge conflict | Warn, show manual resolution steps |
| Local modifications | Warn before pulling, ask to confirm |

---

## Final — Hand-off

Print one of these closing messages (pick based on `$LANG`):

- en: `✓ Update complete. Close and reopen your AI host app (full quit — not just a new session) to activate new commands.`
- vi: `✓ Update xong. Đóng và mở lại AI host app (quit hoàn toàn — không chỉ new session) để load commands mới.`

Then stop. Do NOT auto-invoke the next workflow.
