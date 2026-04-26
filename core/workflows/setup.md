# Workflow: compass:setup

You are the integration manager. Mission: detect, configure, and verify tool connections (Jira, Figma, Confluence, Vercel).

**Principles:** Detect before asking. Verify immediately after setup. Never break the init flow for a restart. Show clear status for every integration.

**Purpose**: View, configure, verify, or reset integrations (Jira, Figma, Confluence, Vercel). Acts as the single entry point for all integration management after init.

**Output**:
- Updates `~/.config/compass/integrations.json` (user-level integration status)
- No project-level files changed (integrations are user-level)

**When to use**:
- Check which integrations are configured: `/compass:setup` (no args)
- Set up a specific one: `/compass:setup <name>` (e.g. `jira`, `figma`, `confluence`, `vercel`)
- Verify all: `/compass:setup verify`
- Verify one: `/compass:setup verify-<name>`
- Reset one: `/compass:setup reset <name>`

---

## Step 0 — Resolve active project

Apply the shared snippet from `core/shared/resolve-project.md`. It sets up `$PROJECT_ROOT`, `$CONFIG`, and `$PROJECT_NAME` for downstream steps and prints the "Using: <name>" banner.

> **Setup-specific fallback**: Setup operates on user-level integrations and can run without an active project. If the resolver returns `status=none` (no Compass project found), skip the ambiguous/none prompts from the shared snippet, default `lang = "en"`, and continue. Show tip at end: "Tip: run /compass:init first to set project preferences."

**Error handling when a project IS resolved** (reading `$PROJECT_ROOT/.compass/.state/config.json` via `$CONFIG`):

1. **$CONFIG missing / status=none**: default `lang = "en"`, continue as described above.

2. **Config exists but is corrupt JSON** (parse error):
   ```
   [Compass] Warning: $PROJECT_ROOT/.compass/.state/config.json could not be parsed (corrupt JSON).
   Defaulting language to English.
   Tip: run /compass:init to reinitialize your project config.
   ```
   Default `lang = "en"`, continue.

3. **Config valid JSON, but missing `lang` field**:
   ```
   [Compass] Warning: config.json is missing the 'lang' field.
   Defaulting language to English.
   ```
   Default `lang = "en"`, continue.

4. **Config valid, `lang` present**: load `lang`, continue.

**Language enforcement**: from this point on, ALL user-facing text MUST use `lang`. JSON keys stay in English.

## Step 1 — Parse arguments

The caller passes an argument string. Parse it into an action:

| Argument | Action |
|---|---|
| (empty / none) | Show status table |
| `<name>` (jira, figma, confluence, vercel) | Delegate to integration setup |
| `verify` | Re-verify ALL configured integrations |
| `verify-<name>` | Re-verify ONE integration |
| `reset <name>` | Mark one as not-configured |

If the argument doesn't match any of the above, show (in `lang`):

```
Unknown argument: <arg>

Usage:
  compass:setup              — show integration status
  compass:setup <name>       — set up an integration (jira, figma, confluence, vercel)
  compass:setup verify       — re-verify all configured integrations
  compass:setup verify-<name> — re-verify one integration
  compass:setup reset <name> — mark one as not-configured
```

## Step 2 — Read current status

Read `~/.config/compass/integrations.json`.

**Error handling — integrations.json:**

1. **File missing**: create it with defaults (see below). No warning needed — first run.

2. **File exists but is corrupt JSON** (parse error):
   - Back up: `integrations.json.backup-<ISO-timestamp>`
   - Create fresh file with defaults
   - Show (in `lang`):
     ```
     [Compass] Warning: integrations.json was corrupt and could not be read.
     A backup was saved to: ~/.config/compass/integrations.json.backup-<timestamp>
     A fresh integrations file has been created.
     Tip: run /compass:init to reconfigure integrations from scratch.
     ```

3. **File exists, valid JSON, but missing required fields** (`version`, `integrations`):
   - Treat as corrupt: back up, create fresh, show same warning as above.

4. **File exists, valid, all fields present**: load and use as-is.

**Default integrations.json structure** (used for missing or corrupt files):

```json
{
  "version": "0.4.0",
  "updated_at": "<now ISO>",
  "integrations": {
    "jira": { "status": "not-configured" },
    "figma": { "status": "not-configured" },
    "confluence": { "status": "not-configured" },
    "vercel": { "status": "not-configured" }
  }
}
```

Ensure `~/.config/compass/` exists: `mkdir -p ~/.config/compass`.

## Step 3 — Execute action

### Action: Show status table (no args)

Display (in `lang`):

```
COMPASS — Integration Status

| Integration | Status              | User          | Details                    |
|-------------|---------------------|---------------|----------------------------|
| Jira        | <status>            | <user or ->   | project: <KEY or ->        |
| Figma       | <status>            | <user or ->   | team: <URL or ->           |
| Confluence  | <status>            | <user or ->   | space: <KEY or ->          |
| Vercel       | <status>            | <mode or ->   | channel: <channel or ->    |

Status legend:
  configured              — working and verified
  configured-pending-verify — installed, needs host restart + verify
  skipped                 — explicitly skipped during init
  not-configured          — never set up
  error                   — last attempt failed (see notes)

Commands:
  /compass:setup <name>         — set up or reconfigure
  /compass:setup verify         — re-verify all
  /compass:setup verify-<name>  — re-verify one
  /compass:setup reset <name>   — clear config (keeps MCP)
```

Format `status` with indicators:
- `configured` → prefix with checkmark
- `configured-pending-verify` → prefix with hourglass
- `skipped` → prefix with dash
- `not-configured` → prefix with dash
- `error` → prefix with warning sign, append `notes` if present

### Action: Delegate to integration setup

Read and follow `~/.compass/core/integrations/<name>.md` in `setup` mode.

Pass the current `lang` and `HOST` detection to the integration workflow.

### Action: Verify all

For each integration where `status` is `configured` or `configured-pending-verify`:
1. Read and follow the corresponding `~/.compass/core/integrations/<name>.md` in `verify` mode.
2. Collect results.

After all verifications, show a summary table:

```
COMPASS — Verification Results

| Integration | Before              | After               | Details        |
|-------------|---------------------|---------------------|----------------|
| Jira        | configured          | configured          | verified OK    |
| Confluence  | pending-verify      | configured          | now verified   |
| Figma       | configured          | error               | 401 bad token  |
| Vercel       | skipped             | (skipped)           | —              |
```

### Action: Verify one

Read and follow `~/.compass/core/integrations/<name>.md` in `verify` mode.

### Action: Reset one

1. Ask the PO to confirm using AskUserQuestion format:

   ```json
   {
     "header": "Reset <Name>",
     "questions": [
       {
         "id": "confirm_reset",
         "question": "This clears Compass's memory of the <Name> integration but does NOT remove the MCP server from your host config. Do you want to continue?",
         "options": ["Yes, reset it", "Cancel"],
         "multiSelect": false
       }
     ]
   }
   ```

   (AI translates per `$LANG` — see `core/shared/ux-rules.md` Language Policy.)

2. If the PO selects "Yes, reset it", overwrite the integration entry:
   ```json
   { "status": "not-configured" }
   ```

3. Atomically write to `~/.config/compass/integrations.json` (read, modify, write to tmp, mv).

4. Confirm (in `lang`):
   ```
   <Name> integration reset to not-configured.
   MCP config at <path> is unchanged — remove manually if needed.

   To set up again: /compass:setup <name>
   ```

5. If the PO selects "Cancel" / "Hủy":
   ```
   Reset cancelled. No changes made.
   ```

---

## Interactive question format (AskUserQuestion)

All questions to the PO during setup or integration flows MUST use this JSON structure:

```json
{
  "header": "<Section or action title>",
  "questions": [
    {
      "id": "<unique_snake_case_id>",
      "question": "<Question text in lang>",
      "options": ["<option1>", "<option2>"],
      "multiSelect": false
    }
  ]
}
```

- `header`: Describe the current step or context (shown as a heading).
- `questions`: Array of question objects. One object per question.
- `id`: Unique identifier for the answer; use snake_case. Used to reference the PO's reply.
- `question`: Full question text, always in `lang`.
- `options`: Predefined choices. MUST have ≥2 options — never use empty array. Provide concrete suggestions; the built-in "Type your own answer" handles custom input.
- `multiSelect`: `true` if the PO may choose more than one option; `false` otherwise.

**Vietnamese example** (general integration selection):
```json
{
  "header": "Cài đặt tích hợp Compass",
  "questions": [
    {
      "id": "select_integration",
      "question": "Bạn muốn thiết lập tích hợp nào?",
      "options": ["Jira", "Figma", "Confluence", "Vercel", "Bỏ qua"],
      "multiSelect": false
    }
  ]
}
```

---

## Save session

Create session record at `$PROJECT_ROOT/.compass/.state/sessions/<ISO-timestamp>-setup/` (skip if `status=none` — no project resolved):
- `transcript.md` — actions taken and results

Never include raw tokens in the transcript.

## Edge cases

- **No config.json (never ran init)**: setup still works — integrations are user-level. Show a note: "Tip: run /compass:init first to set project preferences."
- **integrations.json is corrupted**: back up to `integrations.json.backup-<timestamp>`, create a fresh one, warn the PO. (See Step 2 error handling.)
- **PO runs verify on a not-configured integration**: tell them "This integration hasn't been set up yet. Run /compass:setup <name> to configure it."
- **PO runs reset on something already not-configured**: silently succeed, no error.
- **Multiple hosts**: setup delegates to the integration workflow which handles host detection. Compass doesn't need to handle it here.
- **$PROJECT_ROOT/.compass/.state/sessions/ doesn't exist**: create it (`mkdir -p`).

---

## Final — Hand-off

Print:

`✓ Setup complete. Next: `/compass:init` to set up a project (if you haven't), or `/compass:brief` to start work.`

(AI translates per `$LANG` — see `core/shared/ux-rules.md` Language Policy.)

Then stop. Do NOT auto-invoke the next workflow.
