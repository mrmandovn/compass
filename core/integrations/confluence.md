# Integration: Confluence

**Purpose**: Set up (or verify) Confluence integration for Compass. Confluence lets Compass publish PRDs to the team wiki and read existing knowledge base pages for context.

**Caller**: `/compass:setup confluence`, `/compass:setup verify-confluence`, or the integrations wizard inside `/compass:init` (Phase C).

**Output**:
- Updates `~/.config/compass/integrations.json` — `integrations.confluence`
- May add a server entry to `~/.claude/mcp.json` or `~/.config/opencode/opencode.jsonc`
- Tokens live only in the host MCP config (env var, chmod 600).

**Note**: Confluence MCP is usually NOT pre-installed in either host. Expect to do a full install.

**Shortcut**: if the PO already went through the Jira workflow using `mcp-atlassian`, Confluence is likely already covered — the same server handles both. Check `integrations.jira.mcp_package` first.

---

## Step 0 — Language

Read `.compass/.state/config.json` → `lang` (default `en`).

## Step 1 — Detect host

Same as Jira workflow Step 1 — detect RUNNING host via `claude` CLI check, not directory existence. `~/.claude/` may exist from Compass install without Claude Code being active.

## Step 2 — Parse mode

| Mode | Trigger |
|---|---|
| `setup` | `/compass:setup confluence` or init wizard |
| `verify` | `/compass:setup verify-confluence` |
| `reset` | `/compass:setup reset confluence` |

## Step 3 — Quick check: is Confluence already covered by mcp-atlassian?

Read `~/.config/compass/integrations.json`. If `integrations.jira.mcp_package == "mcp-atlassian"` AND `integrations.jira.status` is `configured` or `configured-pending-verify`:

Try probing Confluence tools:
- `mcp__confluence__confluence_get_user`
- `mcp__atlassian__confluence_search`
- `mcp__jira__confluence_*` (some packages namespace both under `jira`)

If any works → Confluence is already live via the same package. Jump to **Step 4** with `mcp_package = "mcp-atlassian"` (inherit from Jira).

If the tool doesn't exist but `mcp-atlassian` is installed, the env vars for Confluence (`CONFLUENCE_URL`, `CONFLUENCE_USERNAME`, `CONFLUENCE_API_TOKEN`) may not be set. Tell the PO:

```
You have mcp-atlassian configured for Jira.
The same package can handle Confluence, but it's missing Confluence env vars.

Add Confluence credentials to the existing mcp-atlassian entry? (Yes / No, install a separate Confluence MCP)
```

If Yes → skip to **Step 5.3 (collect metadata)** and **Step 5.5 (edit existing entry)**, skipping the package install.

## Step 4 — Already configured path

Reached when probing succeeds.

1. Display:
   ```
   ✓ Confluence is already configured
     User:  <email>
     Base:  <base URL>
     Space: <default_space_key or "—">
   ```

2. If `default_space_key` is not set, ask:
   ```
   Default Confluence space key? (e.g. KMS, SV, ENG — optional, press Enter to skip)
   ```

3. Update `integrations.confluence`:
   ```json
   {
     "status": "configured",
     "configured_at": "<existing or now>",
     "verified_at": "<now ISO>",
     "host": "<HOST>",
     "user": "<email>",
     "base_url": "<URL>",
     "default_space_key": "<KEY or null>",
     "mcp_package": "<package>",
     "notes": ""
   }
   ```

4. Atomic write. Stop.

## Step 5 — Install from scratch (5 sub-steps)

### 5.1 — Explain why (1/5)

```
Compass wants Confluence for:
  • Reading team wiki pages for context when writing PRDs
  • (v0.3) Publishing approved PRDs to your team space
  • Linking stories to design docs and runbooks

Read-only by default. Compass asks before publishing.
Setup takes about 2 minutes.

Note: Confluence and Jira use the same Atlassian account.
If you already set up Jira via mcp-atlassian, your token works here too.

Continue? (Yes / Skip)
```

If Skip → save status `skipped`, return to caller.

### 5.2 — Create API token (2/5)

```
Step 2/5 — Create an Atlassian API token (same as Jira)
  URL: https://id.atlassian.com/manage-profile/security/api-tokens

If you already have a "compass-jira" token, you can reuse it.
If not, create one now and label it "compass-atlassian".
```

Auto-open:

```bash
open "https://id.atlassian.com/manage-profile/security/api-tokens" 2>/dev/null || true
```

Wait for the PO to press Enter.

### 5.3 — Collect metadata (3/5)

Ask:

1. `Confluence base URL? (e.g. https://your-org.atlassian.net/wiki)` — must end with `/wiki` for Atlassian Cloud.
2. `Atlassian account email? (same as Jira)`
3. `API token? (reuse compass-jira if you have one)`
4. `Default Confluence space key? (e.g. KMS, SV, ENG — optional)`

Validate:
- URL starts with `https://` and contains `/wiki` or is a raw `*.atlassian.net` URL (if the latter, append `/wiki` yourself).
- Email contains `@`.
- Token non-empty.
- Space key is uppercase letters/numbers, 2-10 chars (warn if it doesn't match, don't hard-fail).

### 5.4 — Install the MCP server (4/5)

Recommend `mcp-atlassian` (covers both Jira + Confluence).

```
Step 4/5 — Install the MCP server

Recommended: mcp-atlassian (Jira + Confluence in one package)
Alternative: a Confluence-only MCP such as @aashari/mcp-server-atlassian-confluence

Note: if you already installed mcp-atlassian for Jira, Compass will re-use it
and only add Confluence env vars to the existing entry.

Install mcp-atlassian now? (Yes / Skip — already installed / Alternative)
```

If Yes → `npm install -g mcp-atlassian`. Handle errors (npm missing, EACCES, network, 404) the same as the Jira workflow.

If Skip → expect the package to already be installed from the Jira workflow; proceed to the config edit.

### 5.5 — Edit the host MCP config (5/5)

The MCP config schema DIFFERS between Claude Code and OpenCode. Pick the correct branch based on `$HOST` from Step 1. Within each branch there are two cases: new entry (Case A) or merge into existing `jira` entry (Case B — recommended when `mcp-atlassian` already serves Jira).

#### Claude Code branch — `~/.claude/mcp.json`

**Case A — New entry** (no `mcp-atlassian` in config yet):

```json
"confluence": {
  "command": "npx",
  "args": ["-y", "mcp-atlassian"],
  "env": {
    "CONFLUENCE_URL": "<base URL>",
    "CONFLUENCE_USERNAME": "<email>",
    "CONFLUENCE_API_TOKEN": "<token>"
  }
}
```

**Case B — Merge into existing `jira` entry:**

Read the existing `jira` (or `atlassian`) entry. Add the Confluence env vars to its `env` block:

```json
"jira": {
  "command": "npx",
  "args": ["-y", "mcp-atlassian"],
  "env": {
    "JIRA_URL": "...",
    "JIRA_USERNAME": "...",
    "JIRA_API_TOKEN": "...",
    "CONFLUENCE_URL": "<new>",
    "CONFLUENCE_USERNAME": "<new>",
    "CONFLUENCE_API_TOKEN": "<new>"
  }
}
```

Do NOT duplicate the server entry. This is the recommended path when both exist.

#### OpenCode branch — `~/.config/opencode/opencode.jsonc`

JSONC (JSON with comments) — preserve comments when rewriting.

**Case A — New entry** (no `mcp-atlassian` in config yet):

```json
"confluence": {
  "type": "local",
  "enabled": true,
  "command": ["npx", "-y", "mcp-atlassian"],
  "environment": {
    "CONFLUENCE_URL": "<base URL>",
    "CONFLUENCE_USERNAME": "<email>",
    "CONFLUENCE_API_TOKEN": "<token>"
  }
}
```

**Case B — Merge into existing `jira` entry:**

Read the existing `jira` (or `atlassian`) entry. Add the Confluence env vars to its `environment` block:

```json
"jira": {
  "type": "local",
  "enabled": true,
  "command": ["npx", "-y", "mcp-atlassian"],
  "environment": {
    "JIRA_URL": "...",
    "JIRA_USERNAME": "...",
    "JIRA_API_TOKEN": "...",
    "CONFLUENCE_URL": "<new>",
    "CONFLUENCE_USERNAME": "<new>",
    "CONFLUENCE_API_TOKEN": "<new>"
  }
}
```

**Key differences from Claude Code:** `type: "local"` + `enabled: true` are required; `command` is an array; `environment` instead of `env`.

Do NOT duplicate the server entry. This is the recommended path when both exist.

---

Back up before editing. Parse, merge, write, `chmod 600`, show redacted preview.

If the config parse fails, save status `error` with the exact message, give the PO the raw block to paste manually (use the block matching their host).

## Step 6 — Save status + handle restart

```json
{
  "status": "configured",
  "configured_at": "<now ISO>",
  "verified_at": null,
  "host": "<HOST>",
  "user": "<email>",
  "base_url": "<URL>",
  "default_space_key": "<KEY or null>",
  "mcp_package": "mcp-atlassian",
  "notes": ""
}
```

**Try to verify immediately.** If it works → set `verified_at`, show `"✓ Confluence connected!"`. Continue flow.

If verify fails (needs restart):

```
✓ Confluence MCP configured (shared with Jira).

Note: MCP server will be available after you restart <HOST_NAME>.
Run /compass:setup verify-confluence after restart.
```

Use ACTUAL host name. Do NOT break init flow.

## Step 7 — Verify mode

1. Re-probe Confluence tool. Success → `status = configured`, `verified_at = now`, show confirmation with user + space.
2. Failure → troubleshooting table:

   | Symptom | Fix |
   |---|---|
   | Tool not in tool list | Restart your AI host (close and reopen). |
   | 401 | Token invalid. Re-run setup. |
   | 403 on the space | Account lacks access to that Confluence space. Ask Confluence admin. |
   | 404 on base URL | Base URL wrong (probably missing `/wiki`). Re-run setup. |
   | Timeout | Network issue — retry. |

   Mark status `error`.

## Step 8 — Status file shape

Same file: `~/.config/compass/integrations.json`. Only touch `integrations.confluence`. Atomic write. Valid `status` values: `configured`, `configured-pending-verify`, `skipped`, `not-configured`, `error`.

## Save session

Only when invoked from `/compass:setup`. `.compass/.state/sessions/<timestamp>-setup-confluence/transcript.md`. Never log the token.

## Edge cases

- **PO uses Confluence Server (not Cloud)**: base URL is different (e.g. `https://confluence.acme.com`, no `/wiki` suffix), and the auth is a Personal Access Token rather than an API token. Ask "Cloud or Server?" upfront. For Server, link to `https://confluence.acme.com/plugins/servlet/de.tngtech.confluence.plugins.pat/manage`.
- **Space key with lowercase**: Confluence accepts lowercase but most teams use uppercase. Warn but allow.
- **PO already has mcp-atlassian for Jira but Confluence env vars missing**: merge into the existing entry (Case B above). Don't install the package again.
- **Edit failure**: back up, show error, give raw block for manual paste.
- **Token reuse**: explicitly tell the PO they can reuse their Jira token — this saves time and confusion.
