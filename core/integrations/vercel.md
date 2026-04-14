# Integration: Vercel

**Purpose**: Set up Vercel integration for Compass. Vercel lets POs deploy prototypes to preview URLs so stakeholders can review live demos before development.

**Caller**: `/compass:setup vercel`, or the integrations wizard inside `/compass:init` (Phase C).

**Output**:
- Updates `~/.config/compass/integrations.json` — `integrations.vercel`
- May configure Vercel CLI or MCP connection

---

## Step 0 — Language

Read `.compass/.state/config.json` if it exists and load `lang` (default `en`).

**Language enforcement**: all user-facing chat text below uses `lang`.

## Step 1 — Detect host

Check for Vercel CLI:

```bash
if command -v vercel >/dev/null 2>&1; then
  VERCEL_VERSION=$(vercel --version 2>/dev/null)
  echo "VERCEL_CLI_INSTALLED=true"
else
  echo "VERCEL_CLI_INSTALLED=false"
fi
```

## Step 2 — Parse mode

| Mode | Trigger |
|---|---|
| `setup` | `/compass:setup vercel` or init wizard |
| `verify` | `/compass:setup verify-vercel` |
| `reset` | `/compass:setup reset vercel` |

## Step 3 — Check if already configured

If Vercel CLI is installed, try:

```bash
vercel whoami 2>/dev/null
```

- **Success** → integration is live, jump to Step 4.
- **Failure** (not logged in) → jump to Step 5 (install/login).
- **CLI not found** → jump to Step 5.

## Step 4 — Already configured

Display (in `lang`):
```
✓ Vercel is already configured
  User: <username>
  CLI version: <version>
```

Update `integrations.vercel`:
```json
{
  "status": "configured",
  "configured_at": "<now ISO>",
  "verified_at": "<now ISO>",
  "user": "<username>",
  "notes": ""
}
```

Stop.

## Step 5 — Install / Login

### 5.1 — Explain why

Show (in `lang`):

- en:
```
Compass uses Vercel to deploy prototypes so stakeholders can preview live demos.

What you need:
  • A Vercel account (free tier works)
  • Vercel CLI (npm package)

Continue? (Yes / Skip)
```

- vi:
```
Compass dùng Vercel để deploy prototype cho stakeholder xem demo trực tiếp.

Bạn cần:
  • Tài khoản Vercel (miễn phí)
  • Vercel CLI (npm package)

Tiếp tục? (Có / Bỏ qua)
```

Use AskUserQuestion:
```json
{"questions": [{"question": "Set up Vercel for prototype deployments?", "header": "Vercel", "multiSelect": false, "options": [{"label": "Yes, set up now", "description": "Install Vercel CLI and log in (~2 minutes)"}, {"label": "Skip — set up later", "description": "You can configure later with /compass:setup vercel"}]}]}
```

If Skip → save status `skipped`, return to caller.

### 5.2 — Install Vercel CLI

```bash
npm install -g vercel 2>&1
```

Handle errors:
- npm not found → show manual install instructions
- EACCES → suggest `sudo npm install -g vercel` or use npx
- Network error → show offline instructions

### 5.3 — Login

```bash
vercel login
```

This opens the browser for OAuth. Wait for completion.

If login succeeds:
- en: `"✓ Logged in to Vercel as <username>"`
- vi: `"✓ Đã đăng nhập Vercel với tài khoản <username>"`

If login fails → show error, suggest retry.

## Step 6 — Save status

Write `integrations.vercel`:

```json
{
  "status": "configured",
  "configured_at": "<now ISO>",
  "verified_at": "<now ISO>",
  "user": "<username>",
  "cli_version": "<vercel --version>",
  "notes": ""
}
```

## Step 7 — Verify mode

1. Run `vercel whoami`
2. On success: update status = `configured`, show confirmation
3. On failure: show error + troubleshooting:

   | Symptom | Fix |
   |---|---|
   | CLI not found | Run `npm install -g vercel` |
   | Not logged in | Run `vercel login` |
   | Token expired | Run `vercel login` again |

## Save session

Same as other integrations — write to `.compass/.state/sessions/<timestamp>-setup-vercel/transcript.md`.

## Edge cases

- **No npm**: Vercel CLI requires npm. If npm is not installed, skip gracefully with a note.
- **Already logged in via different account**: `vercel whoami` shows current account. Offer to switch.
- **Vercel team vs personal**: Compass works with both. Default to personal scope.
- **Free tier limits**: Note that free tier has 100 deployments/day — more than enough for prototypes.
