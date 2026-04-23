# Shared: Git Context (Branch + Dirty State)

**Purpose**: Detect the repo's base branch, manage the feature branch for a dev session, and handle dirty working-tree state so dev tasks don't accidentally contaminate unrelated work.

**Used by**: `/compass:spec` (Step 1), `/compass:cook` (Step 2), `/compass:fix` (Step 1).

---

## Part A — Detect branching context

```bash
# Resolve the base branch (origin's default, fallback to "main")
BASE_BRANCH=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@')
[ -z "$BASE_BRANCH" ] && BASE_BRANCH=$(git branch --list main master develop 2>/dev/null | grep -E "^[* ] (main|master|develop)$" | head -1 | tr -d '* ')
[ -z "$BASE_BRANCH" ] && BASE_BRANCH="main"

CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null)
DIRTY="no"
[ -n "$(git status --porcelain)" ] && DIRTY="yes"

echo "BASE_BRANCH=$BASE_BRANCH"
echo "CURRENT_BRANCH=$CURRENT_BRANCH"
echo "DIRTY=$DIRTY"
```

If this isn't a git repo at all (`git rev-parse` fails) → print `ℹ Not a git repo — skipping branch management.` and return early. The rest of the workflow still works; dev just doesn't get auto-branch + commit convenience.

---

## Part B — Branch state handling

Given `$SESSION_SLUG`, derive the feat branch name:

```bash
FEAT_BRANCH="feat/$SESSION_SLUG"
# For hotfix sessions, use "fix/" prefix
[ "$IS_HOTFIX" = "true" ] && FEAT_BRANCH="fix/$SESSION_SLUG"
```

Branch on current state:

| `$CURRENT_BRANCH` | `$DIRTY` | `$IS_HOTFIX` | Action |
|---|---|---|---|
| `$BASE_BRANCH` | no | `false` | Create + checkout `$FEAT_BRANCH` via `compass-cli git branch "$FEAT_BRANCH"` or `git checkout -b "$FEAT_BRANCH"`. Print `✓ Created $FEAT_BRANCH from $BASE_BRANCH.` |
| `$BASE_BRANCH` | no | `true` | **AskUserQuestion (option C — hotfix confirmation).** Hotfix flow must always confirm before auto-branching. |
| `$BASE_BRANCH` | yes | any | AskUserQuestion (see below — option A) |
| `$FEAT_BRANCH` (matches session) | no | any | Resume — continue on existing branch. Print `ℹ Continuing on $FEAT_BRANCH.` |
| `$FEAT_BRANCH` (matches session) | yes | any | Resume with WIP. Print `ℹ Continuing on $FEAT_BRANCH (with uncommitted changes).` |
| any other branch | any | any | AskUserQuestion (see below — option B) |

### AskUserQuestion — Option A (on base branch + dirty)

```json
{"questions": [{"question": "You have uncommitted changes on $BASE_BRANCH. How to proceed?", "header": "Dirty state", "multiSelect": false, "options": [
  {"label": "Stash, create branch, work fresh", "description": "git stash push -m \"compass-<slug>\" → create $FEAT_BRANCH → work there. You can pop the stash later."},
  {"label": "Commit first, then branch", "description": "Stop here — you commit current WIP manually, then re-run this workflow."},
  {"label": "Cancel", "description": "Abort — resolve dirty state yourself."}
]}]}
```

vi: translate labels (`Stash, tạo branch`, `Commit trước rồi tạo branch`, `Cancel`).

**Handler — execute ONLY the branch matching the user's selection**:

```bash
case "$DIRTY_CHOICE" in
  stash)
    git -C "$PROJECT_ROOT" stash push -m "compass-$SESSION_SLUG" || {
      echo "✗ Stash failed — working tree may have conflicts. Aborting."; exit 1;
    }
    git -C "$PROJECT_ROOT" checkout -b "$FEAT_BRANCH" || {
      echo "✗ Branch create failed. Stash preserved — 'git stash pop' to recover."; exit 1;
    }
    echo "✓ Stashed as 'compass-$SESSION_SLUG' and created $FEAT_BRANCH."
    ;;
  commit_first)
    echo "ℹ Commit your changes, then re-run /compass:<workflow-name>."
    exit 0
    ;;
  cancel)
    echo "✗ Cancelled."
    exit 0
    ;;
esac
```

### AskUserQuestion — Option B (on unrelated branch)

```json
{"questions": [{"question": "You're on $CURRENT_BRANCH, not the base ($BASE_BRANCH) or this session's branch ($FEAT_BRANCH). How to proceed?", "header": "Branch mismatch", "multiSelect": false, "options": [
  {"label": "Switch to base + create $FEAT_BRANCH", "description": "git checkout $BASE_BRANCH → git checkout -b $FEAT_BRANCH. Any dirty changes will be stashed first."},
  {"label": "Continue on $CURRENT_BRANCH", "description": "Work here. Commits go to this branch. You accept the risk."},
  {"label": "Cancel", "description": "Abort — sort out branching yourself."}
]}]}
```

**Handler — execute ONLY the branch matching the user's selection**:

```bash
case "$MISMATCH_CHOICE" in
  switch_and_create)
    if [ "$DIRTY" = "yes" ]; then
      git -C "$PROJECT_ROOT" stash push -m "compass-$SESSION_SLUG-premigrate" || {
        echo "✗ Stash failed. Aborting."; exit 1;
      }
      echo "✓ Stashed WIP as 'compass-$SESSION_SLUG-premigrate'."
    fi
    git -C "$PROJECT_ROOT" checkout "$BASE_BRANCH" || {
      echo "✗ Checkout to $BASE_BRANCH failed. Resolve manually."; exit 1;
    }
    git -C "$PROJECT_ROOT" checkout -b "$FEAT_BRANCH" || {
      echo "✗ Branch create failed."; exit 1;
    }
    echo "✓ Switched to $BASE_BRANCH and created $FEAT_BRANCH."
    ;;
  continue_here)
    echo "⚠ Continuing on $CURRENT_BRANCH. Commits will land here."
    FEAT_BRANCH="$CURRENT_BRANCH"
    ;;
  cancel)
    echo "✗ Cancelled."
    exit 0
    ;;
esac
```

### AskUserQuestion — Option C (hotfix on clean base — explicit confirm)

Hotfix flow (`IS_HOTFIX=true`) never creates a branch silently. Even when the working tree is clean and we're on the base branch, ask first:

en:
```json
{"questions": [{"question": "Create hotfix branch $FEAT_BRANCH from $BASE_BRANCH?", "header": "Branch", "multiSelect": false, "options": [
  {"label": "Create branch (Recommended)", "description": "git checkout -b $FEAT_BRANCH — isolate the fix for PR + easy revert"},
  {"label": "Stay on $BASE_BRANCH", "description": "Apply fix directly on base branch. Only pick if you know what you're doing."},
  {"label": "Cancel", "description": "Stop the workflow — don't create a branch or apply the fix"}
]}]}
```

vi: translate (`Tạo branch (Khuyến nghị)`, `Ở lại $BASE_BRANCH`, `Huỷ`).

**Handler — execute ONLY the branch matching the user's selection**:

```bash
case "$HOTFIX_CHOICE" in
  create_branch)
    compass-cli git branch "$FEAT_BRANCH" 2>/dev/null \
      || git -C "$PROJECT_ROOT" checkout -b "$FEAT_BRANCH" \
      || { echo "✗ Branch create failed."; exit 1; }
    echo "✓ Created $FEAT_BRANCH from $BASE_BRANCH."
    ;;
  stay_on_base)
    FEAT_BRANCH="$BASE_BRANCH"
    echo "⚠ Applying hotfix directly on $BASE_BRANCH."
    ;;
  cancel)
    echo "✗ Cancelled."
    exit 0
    ;;
esac
```

---

## Part C — Record branch state in session

After Part B resolves, persist to `state.json`:

```bash
# Build JSON safely via jq to avoid quote/escape injection from branch names
# or SHAs that could otherwise corrupt state.json.
GIT_STATE_JSON=$(jq -n \
  --arg base  "$BASE_BRANCH" \
  --arg feat  "$FEAT_BRANCH" \
  --arg bsha  "$(git rev-parse "$BASE_BRANCH" 2>/dev/null)" \
  --arg ssha  "$(git rev-parse HEAD 2>/dev/null)" \
  '{git: {base_branch:$base, feat_branch:$feat, base_sha:$bsha, session_start_sha:$ssha}}')

compass-cli state update "$SESSION_DIR" "$GIT_STATE_JSON"
```

This lets `/compass:cook` verify dev didn't wander off-branch between sessions, and lets `/compass:fix` know the base state for diff calculations.

---

## Part D — Stash recovery

When resuming a session (existing feat branch), check for stashed work created by earlier runs:

```bash
STASH_NAME="compass-$SESSION_SLUG"
STASH_REF=$(git stash list | grep -m1 "$STASH_NAME" | cut -d: -f1)

if [ -n "$STASH_REF" ]; then
  # Notify dev — don't pop silently
  echo "ℹ Found stash \"$STASH_NAME\" ($STASH_REF) from an earlier session."
  # AskUserQuestion: pop / keep stashed / discard
fi
```

AskUserQuestion:

```json
{"questions": [{"question": "Stash $STASH_NAME exists from a previous run. What to do?", "header": "Stash", "multiSelect": false, "options": [
  {"label": "Pop now", "description": "git stash pop $STASH_REF — restore the WIP. Conflict resolution up to you if any."},
  {"label": "Keep for later", "description": "Leave it stashed; you can pop manually with git stash pop $STASH_REF"},
  {"label": "Discard", "description": "git stash drop $STASH_REF — permanently delete the stashed WIP"}
]}]}
```

---

## Part E — Session-end summary

At the end of `/compass:cook` or `/compass:fix`, print a git-state recap:

```bash
COMMIT_COUNT=$(git log --oneline "$BASE_BRANCH..$FEAT_BRANCH" | wc -l | tr -d ' ')
FILES_CHANGED=$(git diff --name-only "$BASE_BRANCH..$FEAT_BRANCH" | wc -l | tr -d ' ')

echo "Branch: $FEAT_BRANCH"
echo "Commits: $COMMIT_COUNT ahead of $BASE_BRANCH"
echo "Files changed: $FILES_CHANGED"
echo ""
echo "Ship when ready:"
echo "  git push -u origin $FEAT_BRANCH"
echo "  gh pr create"
```

---

## Rules

| Rule | Detail |
|---|---|
| **Never force-switch branch** | Always ask via AskUserQuestion when state is ambiguous. |
| **Never discard WIP without confirmation** | Stash by default; discard only via explicit "Discard" option. |
| **Base branch name is fluid** | Silver Tiger might use `main` OR `develop` — detect via origin's HEAD. |
| **Not-a-git-repo is fine** | Don't fail loudly; just skip branch management and continue. |
| **Feat branch = session slug** | `feat/<slug>` or `fix/<slug>` — consistent, derivable. |
| **Don't auto-push** | Session workflows never run `git push`. Dev ships manually. |
