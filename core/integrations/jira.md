# Integration: Jira

**Purpose**: Set up (or verify) Jira integration for Compass. Detects if a Jira MCP server is already configured in the current host; if not, walks the PO through creating an API token, installing the MCP, and editing the host's MCP config file.

**Caller**: `/compass:setup jira`, `/compass:setup verify-jira`, or the integrations wizard inside `/compass:init` (Phase C).

**Output**:
- Updates `~/.config/compass/integrations.json` — `integrations.jira`
- May add a server entry to `~/.claude/mcp.json` (Claude Code) or `~/.config/opencode/opencode.jsonc` (OpenCode)
- Never writes tokens to Compass files. Tokens live only inside the host MCP config as environment variables.

---

## Step 0 — Language

Read `.compass/.state/config.json` if it exists and load `lang` (default `en` if file missing or read fails).

**Language enforcement**: all user-facing chat text below MUST use `lang`. If `lang == vi`, translate every prompt and label. JSON keys stay in English.

## Step 1 — Detect host

Detect which AI host is CURRENTLY RUNNING this workflow (not just which directories exist — `~/.claude/` may exist from Compass adapter install without Claude Code being the active host).

**Detection priority — check the RUNNING host first:**

```bash
HOST="unknown"

# Method 1: Check if Claude Code CLI is available (means CC is installed AND running)
if command -v claude >/dev/null 2>&1; then
  HOST="claude-code"
  MCP_CONFIG="$HOME/.claude/mcp.json"
elif [ -d "$HOME/.config/opencode" ] || [ -d "$HOME/.opencode" ]; then
  HOST="opencode"
  MCP_CONFIG="$HOME/.config/opencode/opencode.jsonc"
fi
```

**IMPORTANT**: The existence of `~/.claude/` directory alone does NOT mean Claude Code is the host. Compass creates that directory for adapter symlinks. Only detect Claude Code if the `claude` CLI binary is actually available.

If HOST = unknown → ask via AskUserQuestion: "Which AI host are you using?" with options "Claude Code" / "OpenCode".

Store `HOST` and `MCP_CONFIG` for use in later steps.

## Step 2 — Parse mode

The caller may invoke this workflow in one of these modes:

| Mode | Trigger | Behavior |
|---|---|---|
| `setup` | `/compass:setup jira` or init wizard | Detect → configure or skip |
| `verify` | `/compass:setup verify-jira` | Re-call detection tool, update `verified_at` |
| `reset` | `/compass:setup reset jira` | Clear status in integrations.json; do NOT remove MCP config |

Default to `setup` if no hint was given.

## Step 3 — Probe for the Jira MCP tool

The Jira MCP server usually exposes a tool named `mcp__jira__jira_get_user_profile`. Check if it's available in the current tool list:

- If the tool is NOT in the tool list at all → Jira MCP is not loaded in this host. Jump to **Step 5 (install from scratch)**.
- If the tool exists in the tool list → call it with no arguments (or with the PO's email if you already know it from a previous run). This verifies the token works.

Call result handling:
- **Success** (returns user profile object with name/email/accountId): the integration is live. Jump to **Step 4 (already configured)**.
- **Failure** (401 unauthorized, 403 forbidden, network error, timeout, "not configured" message): the MCP is loaded but credentials are bad or expired. Show the error text to the PO, then ask: "Re-configure now? (Yes / Skip)". If yes → jump to **Step 5**. If skip → save status `error` with the error message, return to caller.

## Step 4 — Already configured path

You reach this step when `jira_get_user_profile` succeeded.

1. Display (in `lang`):
   ```
   ✓ Jira is already configured
     User:  <display_name> <email>
     Host:  <HOST>
   ```

2. Read the current status from `~/.config/compass/integrations.json`:
   - If the file doesn't exist, create it with an empty structure (see Step 8).
   - If `integrations.jira.default_project_key` is missing OR the PO asked to reconfigure, ask:
     ```
     What is the default Jira project key for this repo? (e.g. SV, KMS, TD)
     Press Enter to skip.
     ```
   - Also ask (only if unset): "What is your Atlassian base URL? (covers Jira + Confluence)". Use AskUserQuestion with auto-detected org URL as first suggestion.

3. Update `integrations.jira` to:
   ```json
   {
     "status": "configured",
     "configured_at": "<existing or now>",
     "verified_at": "<now ISO>",
     "host": "<HOST>",
     "user": "<@handle or email>",
     "url": "<base URL>",
     "default_project_key": "<KEY or null>",
     "mcp_package": "<existing or unknown>",
     "notes": ""
   }
   ```

4. Atomically write the file (read → modify → write).

5. **Return to caller** — do not run install steps. If called from init, init continues to the next integration.

## Step 5 — Install from scratch (5 sub-steps)

You reach this step when the Jira MCP is not loaded or credentials failed.

### 5.1 — Explain why (1/5)

Show (in `lang`):

```
Compass wants Jira for:
  • Searching tickets and epics while writing PRDs and stories
  • Linking generated artifacts to Jira epics/issues
  • (v0.3) Pushing drafted stories to Jira after you confirm

Jira is read-only by default. Compass never pushes without asking.
Setup takes about 3 minutes.

Continue? (Yes / Skip — I'll set up later)
```

If Skip → save status `skipped`, return to caller.

### 5.2 — Create API token (2/5)

Tell the PO to create an Atlassian API token:

```
Step 2/5 — Create an Atlassian API token
  URL: https://id.atlassian.com/manage-profile/security/api-tokens

Instructions:
  1. Open the URL (Compass will try to open it for you)
  2. Click "Create API token"
  3. Label it "compass-jira"
  4. Copy the token immediately (Atlassian only shows it once)

Note: the same token works for Confluence too — save it somewhere secure.
```

Try to auto-open on macOS:

```bash
open "https://id.atlassian.com/manage-profile/security/api-tokens" 2>/dev/null || true
```

On Linux, try `xdg-open`. If the open command fails silently, just print the URL — the PO will copy-paste.

Wait for the PO to press Enter before continuing.

### 5.3 — Collect metadata (3/5)

Ask via AskUserQuestion (in `lang`). Since this setup covers BOTH Jira and Confluence (same Atlassian account), label questions accordingly:

**Question 1 — Atlassian base URL:**

Use AskUserQuestion with smart suggestions. Auto-detect from existing config or common patterns:

Auto-detect org name before showing options:
1. Read git remote URL → extract org (e.g. `github.com/acme-corp/...` → `acme-corp`)
2. Read existing `.env` or config files for Atlassian URLs
3. Read project folder name as fallback

```json
{"questions": [{"question": "Atlassian base URL? (covers both Jira and Confluence)", "header": "Atlassian URL", "multiSelect": false, "options": [
  {"label": "https://<detected-org>.atlassian.net", "description": "Auto-detected from your git remote / project"},
  {"label": "https://<org>.atlassian.net", "description": "Replace <org> with your organization name"}
]}]}
```

Replace ALL `<placeholders>` with ACTUAL detected values before showing to user. NEVER show angle brackets in options.

**Question 2 — Email:** use AskUserQuestion. Build suggestions from:
1. PO name from config + domain from Atlassian URL (e.g. PO="alice", URL="acme.atlassian.net" → suggest `alice@acme.com`)
2. `git config user.email`
3. Domain: extract from the Atlassian URL chosen in Question 1 (e.g. `acme.atlassian.net` → try `acme.com`, `acme.io`, `acme.ae`)

```json
{"questions": [{"question": "Atlassian account email?", "header": "Email", "multiSelect": false, "options": [
  {"label": "<po-name>@<org-domain>", "description": "Based on PO name + organization domain"},
  {"label": "<git-config-email>", "description": "From git config user.email"}
]}]}
```

Replace ALL placeholders with ACTUAL detected values. NEVER show angle brackets to user.
**Question 3 — API token:** ask as follow-up (remind: NOT stored in Compass)
**Question 4 — Default project key:** use AskUserQuestion with detected prefix:

```json
{"questions": [{"question": "Default project key for this repo?", "header": "Project key", "multiSelect": false, "options": [
  {"label": "<PREFIX>", "description": "Detected from project config (e.g. SV, KMS, TD)"},
  {"label": "Skip", "description": "Set later — not required for setup"}
]}]}
```

Strip trailing slash from URL, validate `https://`. Email must contain `@`. Token must be ≥10 chars.

If any field fails validation, re-ask once. If fails twice, save status `error` and return to caller — if called from init, init continues to the next integration.

### 5.4 — Install the MCP server (4/5)

Compass recommends `mcp-atlassian` because it covers both Jira and Confluence in one install.

Ask:

```
Step 4/5 — Install the MCP server

Recommended: mcp-atlassian (covers Jira + Confluence)
Alternative: @modelcontextprotocol/server-atlassian-jira (Jira only)

Install mcp-atlassian via npm? (Yes / Alternative / Skip install — I already have one)
```

If Yes → run:

```bash
npm install -g mcp-atlassian 2>&1
```

Show the full stdout/stderr to the PO. Handle these failure modes:

| Error | Action |
|---|---|
| `command not found: npm` | Tell PO to install Node.js from https://nodejs.org. Save status `error`, note `"npm not available"`, return to caller. |
| `EACCES` permission denied | Suggest `sudo npm install -g mcp-atlassian` OR `npm config set prefix ~/.npm-global`. Ask the PO to retry. |
| Network / registry timeout | Ask PO to check connectivity / proxy. Retry once. If still fails → save status `error`. |
| Package not found (404) | The package name may have moved. Ask PO to pick the alternative. |

On success, capture the package name actually installed (e.g. `mcp-atlassian`) — you'll write it to the status file.

If the PO picks `Skip install`, ask which package they already have and use that string as `mcp_package`.

### 5.5 — Edit the host MCP config (5/5)

Locate the config file:
- Claude Code: `~/.claude/mcp.json`
- OpenCode: `~/.config/opencode/opencode.jsonc`

#### Claude Code branch

1. If `~/.claude/mcp.json` does not exist, create it with `{ "mcpServers": {} }`.
2. Read the file.
3. Parse JSON. If parse fails → back it up to `~/.claude/mcp.json.compass-backup-<timestamp>`, show PO the error, ask them to fix and retry.
4. Merge a new entry under `mcpServers`:

   Key: `jira` (or `atlassian` if the package covers both). If a key with that name already exists, ask the PO: "An entry named `jira` already exists. Overwrite? (Yes / Rename to jira-compass / Cancel)".

   Value shape (exact shape depends on the MCP package — check the package's README. The common shape is):

   ```json
   {
     "command": "npx",
     "args": ["-y", "mcp-atlassian"],
     "env": {
       "JIRA_URL": "<base URL>",
       "JIRA_USERNAME": "<email>",
       "JIRA_API_TOKEN": "<token>",
       "CONFLUENCE_URL": "<base URL>/wiki",
       "CONFLUENCE_USERNAME": "<email>",
       "CONFLUENCE_API_TOKEN": "<token>"
     }
   }
   ```

5. Write the merged JSON back. Preserve any existing servers unchanged.
6. `chmod 600 ~/.claude/mcp.json` — mandatory because the file now contains a token.
7. Show the PO a REDACTED preview of the added block:

   ```json
   "jira": {
     "command": "npx",
     "args": ["-y", "mcp-atlassian"],
     "env": {
       "JIRA_URL": "https://acme.atlassian.net",
       "JIRA_USERNAME": "po@acme.com",
       "JIRA_API_TOKEN": "<REDACTED>",
       ...
     }
   }
   ```

#### OpenCode branch

1. Config file: use `$MCP_CONFIG` from Step 1 (may be `~/.config/opencode/opencode.jsonc` or `~/.opencode/opencode.jsonc`). JSONC (JSON with comments) — be careful not to strip comments when rewriting.
2. If the file doesn't exist, create it with:
   ```json
   {
     "$schema": "https://opencode.ai/config.json",
     "mcp": {}
   }
   ```
3. Find the `mcp` object (OpenCode's MCP schema key). If absent, add it.
4. Merge the server entry keyed `jira` using OpenCode's MCP schema (DIFFERENT from Claude Code — do NOT reuse the block above):

   ```json
   "jira": {
     "type": "local",
     "enabled": true,
     "command": ["npx", "-y", "mcp-atlassian"],
     "environment": {
       "JIRA_URL": "<base URL>",
       "JIRA_USERNAME": "<email>",
       "JIRA_API_TOKEN": "<token>",
       "CONFLUENCE_URL": "<base URL>/wiki",
       "CONFLUENCE_USERNAME": "<email>",
       "CONFLUENCE_API_TOKEN": "<token>"
     }
   }
   ```

   **Key differences from Claude Code's `mcp.json` schema:**
   - `type: "local"` — required discriminator (OpenCode supports multiple server types).
   - `enabled: true` — required flag (OpenCode disables servers by default without it).
   - `command` is an **array** containing the executable + all args (Claude Code separates `command` + `args`).
   - `environment` instead of `env`.

   If a key named `jira` already exists, ask the PO: "An entry named `jira` already exists. Overwrite? (Yes / Rename to jira-compass / Cancel)".

5. Write back. Do not reformat unrelated lines. Preserve comments where possible.
6. `chmod 600 ~/.config/opencode/opencode.jsonc`.
7. Show the PO a REDACTED preview:

   ```json
   "jira": {
     "type": "local",
     "enabled": true,
     "command": ["npx", "-y", "mcp-atlassian"],
     "environment": {
       "JIRA_URL": "https://acme.atlassian.net",
       "JIRA_USERNAME": "po@acme.com",
       "JIRA_API_TOKEN": "<REDACTED>",
       ...
     }
   }
   ```

If config edit fails for any reason (permission denied, parse error, unknown schema), save status `error` with the exact error message and tell the PO to edit the file manually. Give them the raw JSON block to paste.

## Step 6 — Save status + handle restart

After a successful config edit:

1. Update `~/.config/compass/integrations.json` → `integrations.jira`:
   ```json
   {
     "status": "configured",
     "configured_at": "<now ISO>",
     "verified_at": null,
     "host": "<HOST>",
     "user": "<email>",
     "url": "<base URL>",
     "default_project_key": "<KEY or null>",
     "mcp_package": "<package name>",
     "notes": ""
   }
   ```
   NOTE: the token is NOT in this file. Only metadata.

2. **Try to verify immediately** — attempt to call the MCP tool right now. Some hosts reload MCP config without restart.
   - If verify succeeds → mark `verified_at = now`, show `"✓ Jira connected and verified!"`. **Continue the init flow without interruption.**
   - If verify fails (MCP not loaded yet) → mark as `configured` (NOT `pending-verify`), show:

   ```
   ✓ Jira MCP configured.

   Note: The MCP server will be available after you restart <HOST_NAME>.
   But don't worry — Compass will keep working. Just run /compass:setup verify-jira
   after your next restart to confirm.
   ```

   Use the ACTUAL host name: "Claude Code" or "OpenCode" — NEVER use the placeholder `<HOST>` in user-facing text. Read `$HOST` from Step 1.

   **CRITICAL: Do NOT tell the PO to quit and restart NOW. Do NOT break the init flow.** The PO can finish init first, restart later, then verify. Integration works are saved and won't be lost.

## Step 7 — Verify mode

When invoked as `/compass:setup verify-jira`:

1. Re-run Step 3 (probe `mcp__jira__jira_get_user_profile`).
2. On success:
   - Update `integrations.jira.status` = `configured`
   - Update `integrations.jira.verified_at` = now ISO
   - Update `integrations.jira.user` = profile user
   - Show:
     ```
     ✓ Jira verified
       User:    <display_name> <email>
       Project: <default_project_key or "—">
     ```
3. On failure, show the error and a troubleshooting table:

   | Symptom | Fix |
   |---|---|
   | Tool not found in tool list | Host didn't reload MCP config. Restart your AI host (close and reopen). |
   | 401 unauthorized | Token invalid or expired. Re-run `/compass:setup jira` to regenerate. |
   | 403 forbidden | Account lacks permissions on the project. Ask Jira admin. |
   | Network / DNS failure | Check network. Verify base URL typo. |
   | Timeout | Jira Cloud is slow — retry once. If persistent, check corporate proxy. |

   Mark status `error` with the error string. Keep the other metadata intact.

## Step 8 — Status file shape

Path: `~/.config/compass/integrations.json`

If the file doesn't exist yet, create it with:

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

Full shape after Jira is configured:

```json
{
  "version": "0.4.0",
  "updated_at": "2026-04-11T14:32:10Z",
  "integrations": {
    "jira": {
      "status": "configured",
      "configured_at": "2026-04-11T14:30:00Z",
      "verified_at": "2026-04-11T14:32:10Z",
      "host": "claude-code",
      "user": "po@acme.com",
      "url": "https://acme.atlassian.net",
      "default_project_key": "SV",
      "mcp_package": "mcp-atlassian",
      "notes": ""
    },
    "figma": { "status": "not-configured" },
    "confluence": { "status": "not-configured" },
    "vercel": { "status": "not-configured" }
  }
}
```

Valid values for `status`:
- `configured` — MCP loaded and verified
- `configured-pending-verify` — MCP installed but host not yet restarted
- `skipped` — PO explicitly skipped during init/setup
- `not-configured` — never attempted
- `error` — last attempt failed; see `notes` for the error

Atomic write: read → modify in memory → write to `~/.config/compass/integrations.json.tmp` → `mv` over the real path. Never truncate-and-write directly; a crash mid-write would lose other integrations.

Always set `updated_at` to now when writing.

Ensure `~/.config/compass/` exists before writing: `mkdir -p ~/.config/compass`.

## Save session

If the caller is `/compass:init`, the init workflow already writes a session log — no extra session needed here.

If the caller is `/compass:setup`, append a line to `.compass/.state/sessions/<timestamp>-setup-jira/transcript.md` (create the folder) with the actions taken and the final status. Never include the raw token in the transcript.

## Edge cases

- **Both Claude Code and OpenCode installed**: ask the PO which one to target. Advanced POs may run this twice.
- **PO pastes a token with whitespace**: trim before writing.
- **PO's Jira is Jira Server (not Cloud)**: the token flow is different (Personal Access Tokens, not API tokens). Ask once: "Is this Jira Cloud or Jira Server?". For Server, point to `https://<your-jira>/plugins/servlet/de.tngtech.jira.plugins.personalaccesstokens/manage`. Record `jira_type: server` in status.
- **`npm install -g` succeeds but the MCP still fails to load after restart**: likely a `$PATH` issue. Tell the PO to check `which mcp-atlassian`. If empty, their npm global prefix isn't on PATH.
- **PO already has `mcp-atlassian` configured manually**: detection will succeed. Compass will still ask to record the default project key — do NOT overwrite their MCP config.
- **Config file is writable but parsing fails**: always back up before editing, never destructively overwrite.
- **PO runs setup while host is already running**: MCP changes only take effect after a full host restart. Instruct them to fully close and reopen their AI host.
- **Config directory `~/.config/compass` missing**: `mkdir -p` before write.
- **chmod fails (read-only filesystem)**: warn the PO that the file contains a token but permissions could not be locked. Save status `configured-pending-verify` with a note.
