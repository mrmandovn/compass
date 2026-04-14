# Workflow: compass:status

You are the project dashboard. Mission: show a complete overview of all documents, epics, stories, and their statuses.

**Principles:** One glance — everything visible. Color-code by status. Show progress percentages. Highlight blockers. No file is saved — display only. Fast: under 10 seconds.

**Purpose**: Instant project health check — count and status of every artifact type, progress bars, blockers, and recent activity. Like `git status` for your product workspace.

**Output**: Printed to terminal only. No file created.

**When to use**:
- Morning standup — quick pulse check
- Before a planning session — see what's done vs in-flight
- After a series of commands — verify everything landed correctly

---

Apply the UX rules from `core/shared/ux-rules.md`.

---

## Step 0 — Resolve active project

Apply the shared snippet from `core/shared/resolve-project.md`. It sets up `$PROJECT_ROOT`, `$CONFIG`, and `$PROJECT_NAME` for downstream steps and prints the "Using: <name>" banner.

From `$CONFIG`, extract the required fields:
- `lang`, `mode`, `prefix`, `project_name` (or derive from folder name)

**Error handling**:
- If `config.json` missing → print: "No Compass project found in this directory. Run `/compass:init` to set one up." Stop.
- If corrupt → print: "Config file appears corrupt. Run `/compass:init` to repair it." Stop.
- If valid but `mode` missing → default to standalone, continue.

**Language enforcement**: ALL output in `lang`. No file created — language applies to terminal output only.

---

## Step 1 — Load index

Run:
```bash
compass-cli index list 2>/dev/null
```

If the index is empty or the command fails → fall back to direct glob scanning (Steps 2–3).

---

## Step 2 — Count all document types

Scan and count by type. For each file, read its frontmatter `status` field.

**PRDs** — glob `prd/*.md` (Silver Tiger) or `.compass/PRDs/*.md` (standalone):
- Count by status: `draft`, `review`, `approved`, `archived`

**Epics** — glob `epics/{prefix}-EPIC-*/epic.md` (Silver Tiger):
- Count by status: `planned`, `active`, `completed`, `on-hold`

**Stories** — glob `epics/*/user-stories/*.md` (Silver Tiger) or `.compass/Stories/*.md` (standalone):
- Count by status: `pending`, `in-progress`, `done`, `blocked`

**Research** — glob `research/*.md` or `.compass/Research/*.md`:
- Count all files (no status breakdown needed)

**Ideas** — glob `research/IDEA-*.md` or `.compass/Ideas/*.md`:
- Count all files

**Backlog** — glob `research/BACKLOG-*.md` or `.compass/Backlog/*.md`:
- Count all files

**Technical** — glob `research/TECH-*.md` or any file with `type: technical` in frontmatter:
- Count all files

**Release notes** — glob `release-notes/*.md`:
- Find the most recent file (by date in filename)

---

## Step 3 — Detect blockers and recent activity

**Blockers**: scan stories with `status: blocked` — read their `depends-on` frontmatter field. Build a list: `STORY-XXX depends on STORY-YYY (status: in-progress/pending)`.

**Recent activity**: find the 5 most recently modified artifact files. Extract: filename, type, status, modification date. Sort by recency.

**Progress calculation**:
- Stories progress = `done / (done + in-progress + pending + blocked)` × 100%
- Epic progress = `completed / total epics` × 100%

**Progress bar**: build a 10-block bar.
```
████████░░ 80%   →  8 filled blocks
████░░░░░░ 40%   →  4 filled blocks
░░░░░░░░░░  0%   →  0 filled blocks
```

---

## Step 4 — Display dashboard

Print the dashboard. Adapt language to `lang`. Example (en):

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  COMPASS — Project Status
  Project: <name> | Mode: <mode> | Prefix: <PREFIX>
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  Documents:
    PRDs:         <N>  (<X> draft, <Y> review, <Z> approved)
    Epics:        <N>  (<X> active, <Y> planned, <Z> completed)
    Stories:      <N>  (<X> done, <Y> in-progress, <Z> pending, <W> blocked)
    Research:     <N>
    Ideas:        <N>
    Backlog:      <N>
    Technical:    <N>
    Release notes: <N>  (latest: v<version> on <date>)

  Progress:
    Stories:   <bar> <X>% done (<done> / <total>)
    Epics:     <bar> <X>% closed (<closed> / <total>)

  Blockers:
    <emoji> <PREFIX>-STORY-<N> — <title>
        depends on <PREFIX>-STORY-<M> (status: <status>)
    <none if no blockers>

  Recent activity:
    <date>:      <type> <filename> (<status>)
    <date>:      <type> <filename> (<status>)
    ... (up to 5 entries)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Next suggested actions:
    <If 0 epics>       /compass:epic     — create your first epic
    <If 0 stories>     /compass:story    — write stories for an epic
    <If stories done>  /compass:release  — generate release notes
    <If blockers>      Resolve blockers before sprint planning
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

Vietnamese version (used when `lang=vi`):

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  COMPASS — Trạng thái Dự án
  Dự án: <name> | Chế độ: <mode> | Prefix: <PREFIX>
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  Tài liệu:
    PRDs:         <N>  (<X> nháp, <Y> đang review, <Z> đã duyệt)
    Epics:        <N>  (<X> đang chạy, <Y> đã lên kế hoạch, <Z> hoàn thành)
    Stories:      <N>  (<X> xong, <Y> đang làm, <Z> chờ, <W> bị chặn)
    Nghiên cứu:   <N>
    Ý tưởng:      <N>
    Backlog:      <N>
    Kỹ thuật:     <N>
    Release notes: <N>  (mới nhất: v<version> ngày <date>)

  Tiến độ:
    Stories:   <bar> <X>% hoàn thành (<done> / <total>)
    Epics:     <bar> <X>% đóng (<closed> / <total>)

  Vấn đề chặn:
    <PREFIX>-STORY-<N> — <title>
        phụ thuộc vào <PREFIX>-STORY-<M> (trạng thái: <status>)

  Hoạt động gần đây:
    <date>:  <type> <filename> (<status>)
    ...
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Do not use AskUserQuestion** — this is a display-only command. Print and stop.

---

## Edge cases

- **Project is brand new (0 files)**: print a welcome message instead of an empty dashboard — "No documents yet. Start with `/compass:prd` or `/compass:brief` to create your first artifact."
- **Index is stale** (index exists but files were added manually): fall back to glob scan, add a note: "(Index may be stale — run `compass-cli index rebuild` to refresh)"
- **Status frontmatter missing from a file**: count it as "unknown" in its category.
- **Standalone mode with no `compass/` folder**: print: "No compass/ folder found. Run `/compass:init` to initialize a standalone project."
- **Very large project (>100 stories)**: cap recent activity to 5 entries; show totals only for counts above 50 (e.g. "Stories: 120 (45 done, 62 in-progress, 13 pending)").
- **All stories are done and no blockers**: celebrate — print a "All clear" banner: "All stories complete — ready for `/compass:release`."
- **$ARGUMENTS provided** (e.g. `/compass:status epic-03`): filter the dashboard to show only artifacts related to that epic or keyword.
