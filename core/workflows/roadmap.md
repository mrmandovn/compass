# Workflow: compass:roadmap

You are the roadmap strategist. Mission: create a product roadmap from epics, priorities, and timeline constraints.

**Principles:** Visual first — include mermaid gantt. Tie every item to an epic or PRD. Show dependencies between roadmap items. Quarterly view. Never show a roadmap without a priority rationale.

**Purpose**: Generate a quarterly product roadmap that is visual, dependency-aware, and directly linked to existing epics and PRDs in the project.

**Output**: `research/ROADMAP-{PREFIX}-{slug}-{date}.md` (Silver Tiger) or `.compass/Research/ROADMAP-{slug}-{date}.md` (standalone)

**When to use**:
- Preparing for a quarterly planning session
- Leadership needs a visual timeline of what ships when
- You want to show dependencies between epics before sprint planning

---

Apply the UX rules from `core/shared/ux-rules.md`.

---

## Step 0 — Resolve active project

Apply the shared snippet from `core/shared/resolve-project.md`. It sets up `$PROJECT_ROOT`, `$CONFIG`, and `$PROJECT_NAME` for downstream steps and prints the "Using: <name>" banner.

From `$CONFIG`, extract the required fields:
- `lang`, `spec_lang`, `mode`, `prefix`, `output_paths`, `naming`

**Error handling**:
- If `config.json` missing or corrupt → tell user to run `/compass:init`. Stop.
- If valid but missing required fields → list them, ask to run `/compass:init`. Stop.

**Language enforcement**: ALL chat text in `lang`. Artifact in `spec_lang`.

Extract `interaction_level` from config (default: "standard"):
- `quick`: auto-build roadmap from scanned epics + default Q1-Q4 timeline, one review step.
- `standard`: ask timeline, capacity, must-haves vs nice-to-haves, then generate.
- `detailed`: go item by item — confirm placement, dependencies, and resourcing for each epic.

---

## Step 0b — Project awareness check

Apply the shared project-scan module from `core/shared/project-scan.md`.
Pass: keywords=$ARGUMENTS, type="research"

The module handles scanning for existing epics, PRDs, and prior roadmaps. If a prior roadmap exists → ask whether to update it or create a new version.

---

## Step 1 — Scan existing artifacts

1. Glob `epics/{prefix}-EPIC-*/epic.md` — read title, priority, status, and stories count from each.
2. Glob `prd/*.md` (Silver Tiger) or `.compass/PRDs/*.md` (standalone) — read titles and priorities.
3. Check for previous roadmap files in `research/ROADMAP-*.md`.
4. Build an internal list: `[{epic_id, title, priority, status, story_count, dependencies}]`.

Show a summary of what was found before asking questions.

---

## Step 2 — Ask PO about timeline and constraints

Use AskUserQuestion for timeline:

```json
{"questions": [{"question": "What is the planning horizon for this roadmap?\n(Tiếng Việt: Khoảng thời gian lập kế hoạch cho roadmap này là gì?)", "header": "Planning horizon", "multiSelect": false, "options": [{"label": "Q1–Q2 (6 months)", "description": "Near-term planning, 2 quarters / Kế hoạch ngắn hạn, 2 quý"}, {"label": "Q1–Q4 (full year)", "description": "Annual roadmap, all 4 quarters / Roadmap năm, tất cả 4 quý"}, {"label": "Next 3 months only", "description": "Rolling 90-day view / Tầm nhìn 90 ngày liên tục"}, {"label": "Custom — I'll specify dates", "description": "Tell me the exact start and end date / Tôi sẽ chỉ định ngày bắt đầu và kết thúc"}]}]}
```

Use AskUserQuestion for capacity constraints:

```json
{"questions": [{"question": "Any capacity or resource constraints to account for?\n(Tiếng Việt: Có ràng buộc năng lực hoặc nguồn lực nào cần tính đến không?)", "header": "Capacity constraints", "multiSelect": true, "options": [{"label": "Limited dev team (1–3 engineers)", "description": "Small team — fewer parallel tracks / Nhóm nhỏ, ít luồng song song"}, {"label": "Shared resources with another team", "description": "Some epics depend on shared infra or platform team / Một số epic phụ thuộc vào nhóm chia sẻ"}, {"label": "Hard launch deadline", "description": "Specific date we must hit — I'll tell you which epic / Ngày ra mắt cứng — tôi sẽ nói epic nào"}, {"label": "No major constraints", "description": "Plan freely based on priority / Lập kế hoạch tự do theo ưu tiên"}]}]}
```

Use AskUserQuestion for must-haves vs nice-to-haves:

```json
{"questions": [{"question": "Which epics are must-haves for the next release?\n(Tiếng Việt: Epic nào là bắt buộc cho lần phát hành tiếp theo?)", "header": "Must-have epics", "multiSelect": false, "options": [{"label": "All P0 epics are must-haves", "description": "Use priority from epic.md / Dùng mức độ ưu tiên từ epic.md"}, {"label": "I'll specify which epics are locked", "description": "Tell me which ones cannot slip / Tôi sẽ chỉ định epic nào không thể trễ"}, {"label": "Everything is flexible", "description": "Optimize purely by priority and capacity / Tối ưu hoàn toàn theo ưu tiên và năng lực"}]}]}
```

---

## Step 3 — Generate roadmap

Place epics across quarters based on priority (P0 first), dependencies (blockers earlier), and capacity.

**Dependency detection**: if epic B's `epic.md` mentions epic A's ID in Notes or Requirements → B depends on A → A must ship in an earlier or same quarter.

### Mermaid Gantt chart

Derive dates from the timeline chosen in Step 2. Use actual epic IDs and titles from the scan. The structure below is illustrative — replace with real data.

```mermaid
gantt
    title Product Roadmap <YEAR>
    dateFormat  YYYY-MM-DD
    section Q1
        <PREFIX>-EPIC-01 <Title>    :active, epic01, <start>, <end>
        <PREFIX>-EPIC-02 <Title>    :epic02, after epic01, <duration>
    section Q2
        <PREFIX>-EPIC-03 <Title>    :epic03, <start>, <end>
    section Q3
        <PREFIX>-EPIC-04 <Title>    :epic04, <start>, <end>
    section Q4
        <PREFIX>-EPIC-05 <Title>    :epic05, after epic04, <duration>
```

### Quarterly breakdown table

| Quarter | Epic | Priority | Status | Dependencies | Notes |
|---------|------|----------|--------|-------------|-------|
| Q1 | EPIC-01 — Title | P0 | active | — | Must-have for launch |
| Q1 | EPIC-02 — Title | P1 | planned | EPIC-01 | Starts after EPIC-01 ships |
| Q2 | EPIC-03 — Title | P1 | planned | — | |

### Resource allocation notes

- **Q1**: <N> epics, ~<X> story points estimated, <Y> engineers needed
- **Q2**: ...

---

## Step 4 — Review and save

Use AskUserQuestion:

```json
{"questions": [{"question": "Roadmap looks good?\n(Tiếng Việt: Roadmap trông ổn không?)", "header": "Review roadmap", "multiSelect": false, "options": [{"label": "Save the roadmap", "description": "Write file now / Lưu ngay"}, {"label": "Move an epic to a different quarter", "description": "I'll tell you which one / Tôi sẽ chỉ định cái nào"}, {"label": "Add a new item not in epics", "description": "Ad-hoc item to include / Hạng mục bổ sung không có trong epics"}]}]}
```

Save path:
- Silver Tiger: `research/ROADMAP-{PREFIX}-{slug}-{date}.md`
- Standalone: `.compass/Research/ROADMAP-{slug}-{date}.md`

```bash
compass-cli index add "<output-file-path>" "research" 2>/dev/null || true
```

## Save session

`$PROJECT_ROOT/.compass/.state/sessions/<timestamp>-roadmap-<slug>/transcript.md`

## Edge cases

- **No epics exist yet**: build a skeleton roadmap from PRDs instead; note that epics need to be created via `/compass:epic`.
- **All epics are P0**: ask the PO to force-rank them — "if you could only ship two P0 epics this quarter, which two?"
- **Circular dependency detected** (A depends on B, B depends on A): flag immediately via AskUserQuestion, do not generate the gantt until resolved.
- **Hard launch deadline conflicts with P0 epic capacity**: surface the conflict explicitly in the roadmap under "Risks", do not silently squeeze the timeline.
- **More than 10 epics**: split into two gantt charts (H1 and H2) to avoid unreadable output.
- **`spec_lang` is bilingual**: generate both `ROADMAP-...-en.md` and `ROADMAP-...-vi.md`.
- **No PRD linked to an epic**: note that epic as "scope TBD" in the quarterly table — do not invent scope.
