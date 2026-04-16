# Integration: Figma

**Purpose**: Set up (or verify) Figma integration for Compass. Figma gives Compass read-access to design files so PRDs and stories can reference real design context, component names, and screenshots.

**Caller**: `/compass:setup figma`, `/compass:setup verify-figma`, or the integrations wizard inside `/compass:init` (Phase C).

**Output**:
- Updates `~/.config/compass/integrations.json` — `integrations.figma`
- May add a server entry to `~/.claude/mcp.json` or `~/.config/opencode/opencode.jsonc`
- Never stores the Figma token in Compass files. Token lives only in the host MCP config as an env var.

---

## Step 0 — Language

Read `.compass/.state/config.json` if it exists and load `lang` (default `en`).

**Language enforcement**: all user-facing chat text below uses `lang`. JSON keys stay in English.

## Step 1 — Detect host

Same procedure as Jira workflow Step 1 — detect RUNNING host (check `claude` CLI for Claude Code, check opencode directory for OpenCode). Do NOT assume Claude Code just because `~/.claude/` exists — that directory may exist from Compass adapter install only.

## Step 2 — Parse mode

| Mode | Trigger |
|---|---|
| `setup` | `/compass:setup figma` or init wizard |
| `verify` | `/compass:setup verify-figma` |
| `reset` | `/compass:setup reset figma` |

## Step 3 — Probe for the Figma MCP tool

The official Figma MCP from claude.ai exposes `mcp__claude_ai_Figma__whoami`. Community alternatives (e.g. `figma-developer-mcp`) expose `mcp__figma__get_me` or similar. Try each known tool name in order:

1. `mcp__claude_ai_Figma__whoami`
2. `mcp__figma__whoami` / `mcp__figma__get_me`

- If NONE of these tools exist in the tool list → Figma MCP is not loaded. Jump to **Step 5 (install from scratch)**.
- If a tool exists → call it with no arguments. On success, the integration is live. Jump to **Step 4**.
- On failure (401, network, bad token) → show error, ask the PO: "Re-configure now? (Yes / Skip)".

## Step 4 — Already configured path

1. Display (in `lang`):
   ```
   ✓ Figma is already configured
     User: <display_name>
     Email: <email>
     Host: <HOST>
   ```

2. Ask the PO (optional, skip-able):
   ```
   Default Figma team URL for this repo? (optional, press Enter to skip)
   ```

3. Update `integrations.figma`:
   ```json
   {
     "status": "configured",
     "configured_at": "<existing or now>",
     "verified_at": "<now ISO>",
     "host": "<HOST>",
     "user": "<email>",
     "team": "<team URL or null>",
     "mcp_package": "<existing or unknown>",
     "notes": ""
   }
   ```

4. Atomically write.

5. Stop.

## Step 5 — Install from scratch (5 sub-steps)

### 5.1 — Explain why (1/5)

```
Compass wants Figma for:
  • Reading design file context when writing PRDs (component names, screenshots)
  • Linking stories to specific frames and nodes
  • Pulling design tokens and rules into spec files

Read-only. Compass will not modify your Figma files.
Setup takes about 2 minutes.

Continue? (Yes / Skip)
```

If Skip → save status `skipped`, return to caller.

### 5.2 — Create Personal Access Token (2/5)

Try to auto-open the settings page:

```bash
open "https://www.figma.com/settings" 2>/dev/null || xdg-open "https://www.figma.com/settings" 2>/dev/null || start "https://www.figma.com/settings" 2>/dev/null || true
```

Then show the instructions in a formatted block (same style as the integration summary table — indented, clean, with visual markers):

- en:
```
  Figma Personal Access Token

  ① Open figma.com → click your Avatar (top-right)
  ② Settings → Security
  ③ "Personal access tokens" → Generate new token
  ④ Name:        compass-figma
  ⑤ Expiration:  Pick whatever suits you (e.g. 90 days, 1 year, or no expiration)
  ⑥ Scopes:      Enable ALL scopes (full access)
  ⑦ Click "Generate token"
  ⑧ Copy the token NOW — Figma only shows it once!

  Can't find it? Go to: https://www.figma.com/settings
  Then scroll to Security → Personal access tokens
```

- vi:
```
  Tạo Figma Personal Access Token

  ① Mở figma.com → click Avatar (góc phải trên)
  ② Settings → Security
  ③ "Personal access tokens" → Generate new token
  ④ Tên:          compass-figma
  ⑤ Hết hạn:      Chọn theo ý bạn (ví dụ 90 ngày, 1 năm, hoặc no expiration)
  ⑥ Scopes:       Bật TẤT CẢ scopes (full access)
  ⑦ Click "Generate token"
  ⑧ Copy token NGAY — Figma chỉ hiện 1 lần!

  Không tìm thấy? Vào: https://www.figma.com/settings
  Rồi cuộn tới Security → Personal access tokens
```

Wait for PO to confirm they have the token before continuing.

Wait for the PO to confirm they have the token.

### 5.3 — Collect metadata (3/5)

Ask:

1. `Figma account email? (e.g. you@company.com)`
2. `Personal Access Token? (will NOT be stored in Compass, only in the host MCP config)`
3. `Default team URL? (optional, e.g. https://www.figma.com/files/team/123456/Acme)`

Validate:
- Email contains `@`.
- Token is non-empty and looks like a Figma token (starts with `figd_` or is a long alphanumeric string; don't hard-fail on format, just warn if it's < 20 chars). Token must have been created with full scopes enabled.

### 5.4 — Install the MCP server (4/5)

Two common options:

1. **Official claude.ai Figma MCP** — may already ship with Claude Code. If the tool list contained `mcp__claude_ai_Figma__*` earlier but auth failed, the server is present; only the token is missing. In that case, the PO needs to add the token to the existing entry instead of a new package install.
2. **Community `figma-developer-mcp`** — npm package, works on any host.

Ask:

```
Step 4/5 — Install the MCP server

Recommended: figma-developer-mcp (community, works on all hosts)
Alternative: skip install — I already have the claude.ai Figma MCP

Install figma-developer-mcp via npm? (Yes / Skip install)
```

If Yes:

```bash
npm install -g figma-developer-mcp 2>&1
```

Handle errors the same way as the Jira workflow (npm missing, EACCES, network, 404).

If Skip install, ask for the package name the PO wants Compass to reference (e.g. `claude_ai_Figma`) and use that string as `mcp_package`.

### 5.5 — Edit the host MCP config (5/5)

The MCP config schema DIFFERS between Claude Code and OpenCode. Pick the correct branch based on `$HOST` from Step 1.

#### Claude Code branch

Locate `~/.claude/mcp.json`.

1. Back up before editing.
2. Parse.
3. Merge entry keyed `figma` under `mcpServers`:

   ```json
   "figma": {
     "command": "npx",
     "args": ["-y", "figma-developer-mcp"],
     "env": {
       "FIGMA_API_KEY": "<token>"
     }
   }
   ```

   If a key `figma` already exists, ask: "Overwrite? / Rename to figma-compass / Cancel".

4. Write back.
5. `chmod 600 ~/.claude/mcp.json`.
6. Show the PO a redacted preview:

   ```json
   "figma": {
     "command": "npx",
     "args": ["-y", "figma-developer-mcp"],
     "env": { "FIGMA_API_KEY": "<REDACTED>" }
   }
   ```

#### OpenCode branch

Locate `~/.config/opencode/opencode.jsonc` (JSONC — preserve comments).

1. Back up before editing.
2. If the file doesn't exist, create it with:
   ```json
   {
     "$schema": "https://opencode.ai/config.json",
     "mcp": {}
   }
   ```
3. Parse. Find the `mcp` object; create if absent.
4. Merge entry keyed `figma` using OpenCode's MCP schema (DIFFERENT from Claude Code):

   ```json
   "figma": {
     "type": "local",
     "enabled": true,
     "command": ["npx", "-y", "figma-developer-mcp"],
     "environment": {
       "FIGMA_API_KEY": "<token>"
     }
   }
   ```

   **Key differences from Claude Code:** `type: "local"` + `enabled: true` are required; `command` is an array containing executable + args; `environment` instead of `env`.

   If a key `figma` already exists, ask: "Overwrite? / Rename to figma-compass / Cancel".

5. Write back. Preserve comments where possible.
6. `chmod 600 ~/.config/opencode/opencode.jsonc`.
7. Show the PO a redacted preview:

   ```json
   "figma": {
     "type": "local",
     "enabled": true,
     "command": ["npx", "-y", "figma-developer-mcp"],
     "environment": { "FIGMA_API_KEY": "<REDACTED>" }
   }
   ```

If edit fails (either branch), save status `error`, tell the PO to edit manually, give them the raw block for their host.

## Step 6 — Save status + handle restart

Write `integrations.figma`:

```json
{
  "status": "configured",
  "configured_at": "<now ISO>",
  "verified_at": null,
  "host": "<HOST>",
  "user": "<email>",
  "team": "<team URL or null>",
  "mcp_package": "figma-developer-mcp",
  "notes": ""
}
```

**Try to verify immediately** — call the Figma MCP tool. If it works → set `verified_at`, show `"✓ Figma connected and verified!"`. Continue flow.

If verify fails (needs restart):

```
✓ Figma MCP configured.

Note: MCP server will be available after you restart <HOST_NAME>.
Run /compass:setup verify-figma after restart to confirm.
```

Use ACTUAL host name ("Claude Code" or "OpenCode"). Do NOT break the init flow — PO finishes init first, restarts later.

## Step 7 — Verify mode

1. Re-probe the Figma tool (Step 3).
2. On success: update `status` = `configured`, `verified_at` = now ISO, `user` = profile user. Show confirmation.
3. On failure: show error + troubleshooting:

   | Symptom | Fix |
   |---|---|
   | Tool not in tool list | Restart your AI host (close and reopen). |
   | 403 / invalid token | Token expired or revoked. Re-run `/compass:setup figma`. |
   | Timeout | Network issue — retry. |

   Mark status `error`.

## Step 8 — Status file shape

Same file as Jira: `~/.config/compass/integrations.json`. Only touch the `figma` sub-object. Preserve everything else. Atomic write (tmp + mv). Update `updated_at`.

Valid `status` values: `configured`, `configured-pending-verify`, `skipped`, `not-configured`, `error`.

## Save session

Same as Jira — only if invoked via `/compass:setup`, write to `.compass/.state/sessions/<timestamp>-setup-figma/transcript.md`. Never log the token.

## Edge cases

- **Figma free account**: API tokens work on free accounts with read-only scopes — no paid plan required.
- **Token works but file access fails later**: Figma tokens are user-scoped. The PO must have access to the files they want Compass to read. Note this in the final confirmation.
- **No "default project" concept**: Figma work is file-by-file; Compass does not store a single default. The `team` field is optional and only used to suggest starting points.
- **Claude Code bundled Figma MCP already present**: the tool `mcp__claude_ai_Figma__whoami` will work without any install. Compass should detect it and skip the install step entirely.
- **Both MCPs installed at the same time**: that's fine — Compass uses whichever tool responds first. Record the one that succeeded as `mcp_package`.
- **chmod fails**: warn the PO that the file contains a token but permissions could not be locked.
