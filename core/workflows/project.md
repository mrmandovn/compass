# Workflow: compass:project

You are the project switcher. Mission: let the PO see and change which Compass project is active across sessions.

**Principles:** Thin wrapper around `compass-cli project`. Never modify project content — just navigate the registry. Adapt all user-facing text to the PO's `lang` preference (vi/en).

**Purpose**: View all registered Compass projects, or switch which one is active for subsequent `/compass:*` commands.

**Output**: Printed to terminal only. May mutate `~/.compass/projects.json` via the CLI (active pointer + `last_used`).

**When to use**:
- List all registered projects: `/compass:project` or `/compass:project list`
- Switch active project: `/compass:project use <path>`

---

## Step 0 — Parse subcommand

`$ARGUMENTS` contains the subcommand string. Parse:

| Argument | Action |
|---|---|
| (empty / none) | Default to `list` → Step 1 |
| `list` | Step 1 |
| `use <path>` | Step 2 |
| anything else | Print usage + exit cleanly |

**Note**: This workflow is exempt from the shared `resolve-project` Step 0 — it IS the resolver's UI. Load `lang` preference directly from `~/.compass/global-config.json` (if present) for output language; fall back to `en`.

Usage message (AI translates per `$LANG` — see ux-rules Language Policy):

```
Usage:
  /compass:project              — list registered projects (default)
  /compass:project list         — list registered projects
  /compass:project use <path>   — switch active project
```

---

## Step 1 — List all projects

```bash
RESULT=$(compass-cli project list)
```

The CLI returns a JSON array. Each entry has: `name`, `path`, `last_used` (ISO timestamp), `is_active` (bool).

**Empty registry** — if the array is empty, print (AI translates per `$LANG` — see ux-rules Language Policy):

```
No Compass projects registered yet. Run /compass:init to create your first project.
```

Stop.

**Non-empty registry** — format a clean table. For each project:
- Active marker: prefix `* ` if `is_active == true`, otherwise `  ` (two spaces)
- Short name (truncate to 20 chars if longer)
- Path (abbreviate `$HOME` → `~`)
- Relative last-used time: compute from `last_used` — e.g. `just now`, `2h ago`, `yesterday`, `3 days ago`, `2 weeks ago`, `last month`.

Example output (AI translates per `$LANG` — see ux-rules Language Policy):

```
Compass projects:

  * An Empty Place    ~/an-empty-place            (active, last used 2h ago)
    Another Test     ~/One Piece/another-test    (last used yesterday)
    Stealth Note     ~/sn                         (last used 3 days ago)

Switch: /compass:project use <path>
```

Align columns visually. Stop after printing.

---

## Step 2 — Switch active project

If `$ARGUMENTS` begins with `use ` (the space is required), take everything after `use ` as `<path>`.

**Empty path** — if `<path>` is blank after trimming, print (adapt to `lang`) and stop:

```
Missing path. Usage: /compass:project use <path>
Tip: run /compass:project list to see registered projects.
```

**Run the CLI** (accepts relative or absolute path; CLI resolves + normalizes):

```bash
compass-cli project use "<path>"
```

**On success** — CLI prints the resolved name + path. Echo a confirmation (AI translates per `$LANG` — see ux-rules Language Policy):

```
✓ Active project: <name> (<path>)
```

If the CLI auto-added the project (path had a valid `.compass/.state/config.json` but wasn't in the registry yet), append: `  (auto-added to registry)`.

**On error** — CLI exits non-zero with a human-readable message on stderr. Surface it and suggest next step (AI translates per `$LANG` — see ux-rules Language Policy):

```
✗ Cannot switch: <error message>
  Tip: run /compass:init in that directory first, or /compass:project list to see valid paths.
```

Stop cleanly — never bubble a raw stack trace up to the PO.

---

## Rules

| Rule | Detail |
|------|--------|
| Read-only for project content | `/compass:project` never touches PRDs, stories, epics, or any artifact. Only the registry + active pointer. |
| Thin wrapper | All real logic lives in `compass-cli project`. This workflow only parses args, formats output, and surfaces errors. |
| Auto-add on use | If `<path>` has a valid Compass config but isn't in the registry, the CLI auto-adds it. The workflow mentions this in the confirmation. |
| `lang` preference | All user-facing strings adapt to the global `lang` (en/vi). JSON payloads from the CLI stay in English. |
| Exit cleanly | Every error path prints a clear message + a hint for what to do next. Never crash silently. |
| Exempt from resolve-project | This IS the switcher — it must not recursively call the resolver. |

---

## Edge cases

- **Path contains spaces** (e.g. `/Users/mando/One Piece/foo`) — always quote `"<path>"` when invoking the CLI.
- **Path is relative** (e.g. `./foo` or `../bar`) — CLI handles resolution; pass through as-is.
- **`compass-cli` not on PATH** — bash will fail; print: "compass-cli not found. Run the Compass installer or add ~/.compass/cli/bin to PATH."
- **Registry file corrupt** — CLI returns a clear error; surface it and suggest `/compass:init` or manual inspection of `~/.compass/projects.json`.
- **Active project was deleted from disk** — `list` still shows it (registry is the source of truth); the next `/compass:*` command will handle the `ambiguous` / `none` case via `resolve-project.md`.
