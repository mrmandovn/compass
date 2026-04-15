# Workflow: compass:ideate

You are the creative facilitator. Mission: turn a pain point into 5-10 diverse, actionable ideas with impact/effort assessment.

**Principles:** Quantity before quality — generate breadth first, then score. Challenge assumptions. Include at least one unconventional idea. Constraints are inputs, not blockers. ≥3 options when asking PO to choose direction or scope.

**Purpose**: Structured brainstorming — turn one pain point / opportunity / piece of feedback into a set of ideas with pros and cons. Avoids "drifting brainstorm".

**Output**:
- Silver Tiger mode: `{output_paths.ideas}/{naming.idea resolved}` — e.g. `research/IDEA-{PREFIX}-{slug}-{date}.md`
- Standalone mode: `{naming.idea_standalone resolved}` — e.g. `.compass/Ideas/IDEA-{slug}-{date}.md`

**When to use**:
- You have user feedback / a complaint and don't yet know how to solve it
- You have a business goal (e.g. "+20% activation") but no feature in mind
- You need to explore the solution space before committing to one direction

---

Apply the UX rules from `core/shared/ux-rules.md`.

> **Additional rule for ideate**: When you truly cannot suggest any context-aware options, describe 2–3 concrete EXAMPLES as options (e.g. "e.g. users complain upload is slow", "e.g. competitor X launched feature Y") rather than leaving options empty.

---

## Step 0 — Resolve active project

Apply the shared snippet from `core/shared/resolve-project.md`. It sets up `$PROJECT_ROOT`, `$CONFIG`, and `$PROJECT_NAME` for downstream steps and prints the "Using: <name>" banner.

From `$CONFIG`, extract the required fields:
- `lang` — chat language (`en` or `vi`)
- `spec_lang` — language for the artifact file (`same` | `en` | `vi` | `bilingual`). When `same`, resolve to `lang`.
- `mode` — `silver-tiger` or `standalone`
- `prefix` — project prefix (Silver Tiger only)
- `output_paths` — where to write artifacts
- `naming` — filename patterns

**Error handling:**
- If the file does not exist: tell the user to run `/compass:init` first, then stop.
- If the file exists but cannot be parsed as valid JSON (corrupt, truncated, encoding issue): tell the user `$PROJECT_ROOT/.compass/.state/config.json` appears to be corrupt and ask them to run `/compass:init` to regenerate it, then stop. Do NOT attempt to guess at field values from a broken file.
- If the file parses but is missing required fields (`lang`, `mode`, or `output_paths`): list the missing fields explicitly, tell the user to run `/compass:init`, then stop.

**Naming resolution** (read before Steps 5–6):
- Silver Tiger output filename: use `naming.idea` from config if present; fallback to `IDEA-{PREFIX}-{slug}-{YYYY-MM-DD}.md`.
- Standalone output filename: use `naming.idea_standalone` from config if present; fallback to `IDEA-{slug}-{YYYY-MM-DD}.md`.
- Resolve `{PREFIX}`, `{slug}`, `{YYYY-MM-DD}` as runtime substitutions.

**Language enforcement**: ALL chat text in `lang`. Artifact file in `spec_lang`. Don't switch mid-conversation.

Extract `interaction_level` from config (default: "standard" if missing):
- `quick`: minimize questions — auto-fill defaults, skip confirmations, derive everything from context. Only ask when truly ambiguous.
- `standard`: current behavior — ask key questions, show options, confirm decisions.
- `detailed`: extra questions — deeper exploration, more options, explicit confirmation at every step.

---

### Auto mode (interaction_level = quick)

If interaction_level is "quick":
1. Derive trigger and description directly from $ARGUMENTS — skip Question 1 (trigger) and Question 2 (description) AskUserQuestion calls.
2. Skip Question 3 (constraints) — assume no constraints unless $ARGUMENTS mentions any.
3. Proceed immediately to Step 3 and generate ideas.
4. Show results for one final review: "OK? / More ideas?"
5. Total questions: 0-1 (only the final review)

If interaction_level is "detailed":
1. Run Step 2 as normal with all three questions
2. Before generating ideas, add extra questions about constraints, scoring criteria, and idea categories
3. After generating ideas, ask which categories to expand
4. Total questions: ~8-12

If interaction_level is "standard":
1. Current behavior — no changes needed

---


## Step 0a — Pipeline + Project choice gate

This workflow produces an artifact in the project, so apply Step 0d from `core/shared/resolve-project.md` after Step 0. The shared gate:

- Scans all active pipelines in the current project and scores their relevance to `$ARGUMENTS`.
- Asks one case-appropriate question (continue pipeline / standalone here / switch to another project / cleanup hint).
- Exports `$PIPELINE_MODE` (true/false), `$PIPELINE_SLUG` (when true), and a possibly-updated `$PROJECT_ROOT` (if the PO picked another project).

After Step 0a returns:
- If `$PIPELINE_MODE=true` → when writing artifacts later, also copy into `$PROJECT_ROOT/.compass/.state/sessions/$PIPELINE_SLUG/` and append to that `pipeline.json` `artifacts` array.
- If `$PROJECT_ROOT` changed → re-read `$CONFIG` and `$SHARED_ROOT` from the new project before proceeding.

---

## Step 0b: Project awareness check

Apply the shared project-scan module from `core/shared/project-scan.md`.
Pass: keywords=$ARGUMENTS, type="idea"

The module handles scanning, matching, and asking the user:
- If "Ideate within PRD scope" → read the PRD (and related files), constrain the brainstorm to that product scope
- If "Ideate something new" → continue normal flow without PRD constraints
- If "Update" → read the existing idea file, ask what needs changing, update in place
- If "New version" → read existing idea as base, bump version, create new
- If "Show me" → display files, re-ask

---

## Step 1 — Read context

1. Load team, PO, and stakeholders from `$CONFIG` (already parsed in Step 0).
2. List existing ideas and PRDs:
   - Silver Tiger: scan `research/IDEA-*.md` and `prd/*.md`
   - Standalone: scan `.compass/Ideas/` and `.compass/PRDs/`
3. **Silver Tiger only — load capability registry:**
   - Read `{capability_registry}` (path from config, e.g. `../shared/capability-registry.yaml`).
   - If the file does not exist or is unreadable, log a warning internally and continue without cross-product context (non-fatal).
   - Store the capability list in memory for use in Step 3 (idea diversity) and Step 4 (self-review for cross-product risks).

## Step 2 — Ask the user about the starting point

Ask the 3 questions below using AskUserQuestion. On mid-tier models, ask one at a time; on frontier models, you may bundle all three.

### Question 1: Trigger

Use AskUserQuestion with:
- `header`: `"What's sparking this brainstorm?"` (en) / `"Điều gì khởi động buổi brainstorm này?"` (vi)
- `question`: `"Where does this idea come from?"` (en) / `"Ý tưởng này xuất phát từ đâu?"` (vi)
- `multiSelect`: false
- `options`:
  - `{label: "User / customer pain point", description: "A frustration or unmet need reported by users"}` (en) / `{label: "Điểm đau của người dùng / khách hàng", description: "Một nỗi bực bội hoặc nhu cầu chưa được đáp ứng từ người dùng"}` (vi)
  - `{label: "Specific feedback or complaint", description: "A concrete piece of feedback received"}` (en) / `{label: "Phản hồi hoặc khiếu nại cụ thể", description: "Một phản hồi cụ thể đã nhận được"}` (vi)
  - `{label: "Business goal to hit", description: "A metric, revenue, or activation target we need to reach"}` (en) / `{label: "Mục tiêu kinh doanh cần đạt", description: "Một chỉ số, doanh thu hoặc mục tiêu kích hoạt cần đạt"}` (vi)
  - `{label: "Something a competitor does well", description: "A feature or UX pattern worth exploring"}` (en) / `{label: "Điều đối thủ đang làm tốt", description: "Một tính năng hoặc UX đáng khám phá"}` (vi)
  - `{label: "Spontaneous idea", description: "No clear external source — just an idea worth exploring"}` (en) / `{label: "Ý tưởng tự phát", description: "Không có nguồn bên ngoài rõ ràng — chỉ là ý tưởng đáng khám phá"}` (vi)

### Question 2: Short description

**Before asking**: scan the project for context to generate smart suggestions:
1. Read recent PRDs (`prd/*.md`) — extract open questions or "Problem statement" sections
2. Read existing IDEAs (`research/IDEA-*.md` or `.compass/Ideas/IDEA-*.md`) — avoid duplicating
3. Read capability-registry.yaml (Silver Tiger) — find gaps or missing capabilities
4. Check the trigger chosen in Question 1 to tailor examples

Use AskUserQuestion with ≥2 context-aware suggestions. Each option is a concrete problem/opportunity description that the PO can select or modify:

- `header`: `"Describe the problem"` (en) / `"Mô tả vấn đề"` (vi)
- `question`: `"What's the problem or opportunity? Pick one below or write your own."` (en) / `"Vấn đề hoặc cơ hội là gì? Chọn gợi ý bên dưới hoặc tự viết."` (vi)
- `multiSelect`: false
- `options`: Generate 2–3 options dynamically based on project context. Examples:

  **If trigger = "User pain point" and existing PRDs mention auth issues:**
  - `{label: "Auth flow quá phức tạp", description: "Người dùng mất 5 bước để đăng nhập, tỉ lệ drop-off 40% ở bước MFA"}`
  - `{label: "Onboarding mới chưa rõ ràng", description: "User mới không biết bắt đầu từ đâu, support ticket tăng 30%"}`

  **If trigger = "Business goal" and no existing context:**
  - `{label: "Tăng activation rate", description: "Ví dụ: 'activation rate hiện tại 25%, mục tiêu Q2 là 40%'"}`
  - `{label: "Giảm churn", description: "Ví dụ: 'monthly churn 8%, muốn giảm xuống 5% bằng cách cải thiện retention'"}`
  - `{label: "Mở rộng thị trường mới", description: "Ví dụ: 'cần hỗ trợ tiếng Nhật để mở rộng sang thị trường Japan'"}`

  **Fallback (no project context available):**
  - `{label: "Ví dụ: Upload file chậm và hay lỗi", description: "Người dùng phản ánh upload >100MB thường bị ngắt, mất 4 lần click"}`
  - `{label: "Ví dụ: Dashboard thiếu metric quan trọng", description: "Team sales cần thấy MRR, churn rate nhưng dashboard chỉ có DAU"}`
  - `{label: "Ví dụ: Mobile app chưa có offline mode", description: "Khách hàng field service cần truy cập data khi không có internet"}`

**Key rule**: options phải là CỤ THỂ, không phải "Open text" hay "Nhập mô tả". PO chọn 1 gợi ý làm base rồi sửa, hoặc tự viết qua "Type your own answer".

### Question 3: Constraints

Use AskUserQuestion with:
- `header`: `"Any constraints to keep in mind?"` (en) / `"Có ràng buộc nào cần lưu ý không?"` (vi)
- `question`: `"Select all that apply, or skip if none."` (en) / `"Chọn tất cả những gì phù hợp, hoặc bỏ qua nếu không có."` (vi)
- `multiSelect`: true
- `options`:
  - `{label: "Tight timeline (≤2 weeks)", description: "Solution must ship within 2 weeks"}` (en) / `{label: "Timeline gấp (≤2 tuần)", description: "Giải pháp phải ra mắt trong vòng 2 tuần"}` (vi)
  - `{label: "No backend changes", description: "Frontend-only solution required"}` (en) / `{label: "Không thay đổi backend", description: "Chỉ được thay đổi frontend"}` (vi)
  - `{label: "No additional infra cost", description: "Must use existing infrastructure"}` (en) / `{label: "Không thêm chi phí hạ tầng", description: "Phải dùng hạ tầng hiện có"}` (vi)
  - `{label: "Must work on mobile", description: "Solution must be mobile-compatible"}` (en) / `{label: "Phải hoạt động trên mobile", description: "Giải pháp phải tương thích mobile"}` (vi)
  - `{label: "No new external dependencies", description: "Cannot add third-party services or libraries"}` (en) / `{label: "Không thêm phụ thuộc bên ngoài", description: "Không được thêm dịch vụ hoặc thư viện bên thứ ba"}` (vi)
  - `{label: "Other / None", description: "Skip or add a constraint in chat"}` (en) / `{label: "Khác / Không có", description: "Bỏ qua hoặc thêm ràng buộc trong chat"}` (vi)

## Step 3 — Generate ideas

Produce **5–10 solution ideas** in the format below. Each idea should be a *distinct approach* — NOT a small variation of the same direction.

Diversity rules — at least one of each:
- A "do nothing / just educate users" idea
- A "fix it properly, expensive" idea
- A "quick win, 80/20" idea
- A "lateral thinking — change the flow / mental model entirely" idea
- A "leverage something we already have from another feature" idea

**Silver Tiger only — cross-product awareness**: When generating ideas, consult the capability registry loaded in Step 1. If any idea would naturally touch a capability owned by another product team, note it in `**Open questions**` as: `"Depends on [Capability Name] — coordinate with [PO]?"`. This is informational; do NOT block idea generation on it.

Format for each idea:

```markdown
### Idea N: <Short, action-oriented name>

**Summary**: One sentence describing what this idea does.

**Who benefits**: <specific user segment>

**Effort estimate**: S / M / L / XL
**Impact estimate**: Low / Medium / High

**Pros**:
- <pro 1>
- <pro 2>

**Risks / trade-offs**:
- <risk 1>
- <risk 2>

**Open questions**: 1–2 things to validate before committing
```

## Step 4 — Self-review

After generating, the model self-reviews:
1. Are any ideas essentially duplicates? Merge if so.
2. Does any idea violate a stated constraint? Drop it or mark `⚠️ violates constraint X`.
3. Are the 5–10 ideas genuinely diverse in approach? If not, add more.
4. **Silver Tiger only**: For each idea touching a capability in the registry, verify the cross-product note was added to `**Open questions**`. If a dependency was missed, add it now.

## Step 5 — Write the output file

Slug = kebab-case of the topic, e.g. `upload-flow-friction`.

**Filename resolution:**
- Silver Tiger: resolve `naming.idea` from config (substituting `{PREFIX}`, `{slug}`, `{YYYY-MM-DD}`). If `naming.idea` is absent, use `IDEA-{PREFIX}-{slug}-{YYYY-MM-DD}.md`. Write to `{output_paths.ideas}/`.
- Standalone: resolve `naming.idea_standalone` from config (substituting `{slug}`, `{YYYY-MM-DD}`). If `naming.idea_standalone` is absent, use `IDEA-{slug}-{YYYY-MM-DD}.md`. Write to `.compass/Ideas/`.

Create the parent folder if it doesn't exist (`mkdir -p`).

File structure:

```markdown
---
title: <Topic name>
created: <YYYY-MM-DD>
po: <from config>
trigger: <from Step 2 Q1>
status: brainstorm
---

# IDEA: <Title>

## Context

<Description from Step 2 Q2>

## Constraints

<List from Step 2 Q3>

## Ideas

<5–10 ideas from Step 3>

## Next steps

- [ ] Review with <relevant stakeholders from config>
- [ ] Pick top 2–3 ideas → /compass:prioritize to score
- [ ] Whichever passes → /compass:prd to spec in detail
```

If `spec_lang` is `bilingual`, also generate a translated version alongside the primary file (e.g. `IDEA-foo-2026-04-11.md` in English + `IDEA-foo-2026-04-11-vi.md` in Vietnamese).

**After writing the file, update the project index:**
```bash
compass-cli index add "<output-file-path>" "idea" 2>/dev/null || true
```
This keeps the index fresh for the next workflow — instant, no full rebuild needed.

## Step 6 — Discover cross-product dependencies (Silver Tiger mode only)

This step is SKIPPED in standalone mode.

1. The capability registry was already loaded in Step 1. Use it now for a final dependency pass.
2. Extract keywords from the ideas generated in Step 3 (feature names, user flow mentions, problem statement).
3. For each capability in the registry, check if the idea keywords match any of its `keywords` list.
4. For each matched capability:
   - Record: name, product, domain, po, po_lead.
   - Mark as `direct` dependency.
5. Follow transitive dependencies ONE level deep:
   - For each directly matched capability, check its `consumers` list.
   - If the current product appears in `consumers`, mark the relationship.
   - Check the matched capability's own likely dependencies and mark as `transitive via <capability>`.
6. Append a Dependencies section to the output file:

```markdown
## Cross-product Dependencies
| Capability | Type | Domain | PO | PO Lead |
|---|---|---|---|---|
| <Capability Name> | direct | <domain> | <@po> | <@po_lead> |
| <Capability Name> | transitive (via <X>) | <domain> | <@po> | <@po_lead> |
```

If no dependencies are found, omit the section entirely (do NOT write an empty table).

## Step 7 — Confirm with the user

Show a summary in `lang`:

```
✓ Generated <N> ideas for "<topic>"
  File: <resolved output path>

Top 3 by Impact / Effort:
  1. <Idea X> — Impact High / Effort S
  2. <Idea Y> — Impact High / Effort M
  3. <Idea Z> — Impact Medium / Effort S

Next:
  /compass:prioritize  — full scoring if you want to compare
  /compass:prd <idea>  — write a PRD for one specific idea
```

## Save session

`$PROJECT_ROOT/.compass/.state/sessions/<timestamp>-ideate-<slug>/transcript.md`

## Edge cases

- **User describes the problem vaguely**: ask once more with a concrete example prompt using AskUserQuestion. If still vague after the second attempt, generate ideas based on explicit assumptions and mark `> **Assumption**: …` at the top of the Context section.

- **A recent IDEA file already exists for this topic (≤7 days)**: warn the user and use AskUserQuestion to ask whether to (a) read the old file first and continue from it, or (b) create a new file with suffix `-v2`.

- **All ideas violate stated constraints**: do NOT silently drop all ideas and produce an empty file. Instead, present the constraint conflicts via AskUserQuestion and ask the user to either (a) relax one or more constraints, or (b) keep the ideas marked `⚠️ violates constraint X` as exploratory options. Never output a file with zero ideas.

- **User doesn't know the difference between ideas**: if two or more ideas generated in Step 3 are functionally indistinguishable to a non-technical stakeholder, add a `**Why this differs from Idea N**` line to each affected idea before writing the file. This is automatic — no user prompt needed.

- **XL-sized ideas present**: if any idea is estimated `Effort: XL`, surface it first in the Step 7 confirmation summary (above High-impact / Low-effort ideas) with a note: `⚡ XL — consider prioritizing for planning lead time`. Do NOT reorder ideas inside the file itself; reorder only in the Step 7 chat summary.

- **Duplicate detection across sessions**: before generating, scan existing IDEA files (as loaded in Step 1). If a past idea file covers ≥80% of the same topic (match on slug similarity or keyword overlap), warn the user with the file path and date, then use AskUserQuestion to ask whether to (a) open the existing file instead, (b) build on it (append a new round of ideas to the same file), or (c) create a fresh file anyway.

- **Empty or vague problem description that cannot be improved**: if the user's Q2 answer is fewer than 10 words OR contains no concrete nouns (no product name, user segment, or measurable symptom), automatically follow up with AskUserQuestion before proceeding to Step 3. Offer three prompt starters as options: (a) a who/what/when/impact template, (b) a user quote format (`"[User type] said: '…'"`) (c) a metrics frame (`"We want to improve [metric] from X to Y"`). Use whatever the user provides — even if still brief — and mark it `> **Note**: Problem description is brief; ideas are based on best-effort interpretation.`
