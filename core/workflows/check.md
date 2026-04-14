# Workflow: compass:check

You are the quality gate. Mission: validate all Colleague outputs for consistency, completeness, and correctness.

**Principles:** Check every cross-reference. Flag TBDs without owners. Verify naming conventions. Never auto-approve — show the report and let PO decide.

**Purpose**: Run full cross-document validation on Compass outputs. Check naming consistency, cross-references, TBDs, traceability. Optionally push results to Jira/Confluence.

**Input**: Latest completed run session
**Output**: Validation report + optional external push

---

Apply the UX rules from `core/shared/ux-rules.md`.

> **Additional rule for check**: Show progress implicitly (e.g. "Almost done — checking traceability") not explicitly ("Step 3 of 5").

---

## Step 0 — Resolve active project

Apply the shared snippet from `core/shared/resolve-project.md`. It sets up `$PROJECT_ROOT`, `$CONFIG`, and `$PROJECT_NAME` for downstream steps and prints the "Using: <name>" banner.

From `$CONFIG`, extract: `lang`, `spec_lang`, `prefix`, `naming`, `output_paths`, `integrations_override`. If missing → tell user to run `/compass:init` first and stop.

All output from this point is in `lang`.

**Vietnamese prompt examples:**
- "Kiểm tra tất cả tài liệu đầu ra — cross-references, naming, TBD, traceability."
- "Đẩy kết quả lên Jira sau khi validate xong."
- "Tìm TBD không có owner trong PRD và Story."

---

## Step 1: Load session

Scan `$PROJECT_ROOT/.compass/.state/sessions/` for the latest folder containing a `result.json` with `"status": "completed"`.

Collect all output file paths listed in the session's `result.json`.

If no completed session found → show (in `lang`):
- en: `"No completed run found. Run /compass:run first, then come back here."`
- vi: `"Chưa có run nào hoàn thành. Chạy /compass:run trước rồi quay lại."`
Then stop.

**Pipeline detection (alongside run session):**

Also check whether the same session directory contains a `pipeline.json` with `"status": "active"`. If found:
- Set `pipeline_active = true` and store the `pipeline.json` path.
- Read and store the full `artifacts` array from `pipeline.json` for use in validation and final summary.
- These pipeline artifacts are validated in addition to the `result.json` outputs.

---

## Step 2: Cross-document validation (parallel)

**Run all 6 checks in parallel** — spawn one agent per check type. Do NOT run sequentially.

```
Running 6 validation checks in parallel:

  🔍 Cross-references      Checking REQ links between PRD ↔ Stories
  🔍 Naming conventions    Verifying file names match config.naming patterns
  🔍 TBD audit             Scanning all docs for orphan TBDs
  🔍 Traceability matrix   Mapping PRD requirements → Stories → ACs
  🔍 Template compliance   Checking required sections are filled
  🔍 Language consistency   Verifying all docs use spec_lang
```

Each agent receives: the list of output files from Step 1 + the specific check to perform. Each returns: `{check: "name", status: "pass|warn|fail", details: [...]}`.

Merge all 6 results into a single report before showing to PO.

Collect pass/warn/fail per check:

### Check 1: Cross-references

- Every `[REQ-xx]` referenced in any Story must exist in a PRD in the session.
- Every Story must reference a valid PRD filename (check front-matter or header).
- Flag any dangling cross-references to non-existent docs.

### Check 2: Naming consistency

- All output files must match `config.naming` patterns (e.g. `{PREFIX}-{YYYY-MM-DD}-{slug}.md`).
- Prefix must match `config.prefix`.
- Dates must be `YYYY-MM-DD` format.
- Report each file that violates naming conventions.

### Check 3: TBD audit

- Scan all output files for strings: `"TBD"`, `"TODO"`, `"TBC"`.
- Each TBD must have an owner assigned (e.g. `TBD @alice` or `TODO: owner=bob`).
- Report all orphan TBDs (no owner) with file path + line context.

### Check 4: Traceability matrix

- Build matrix: PRD Requirement → Story → Acceptance Criteria.
- Flag every requirement that has no Story covering it.
- Flag every Story that has no Acceptance Criteria section.
- Flag every Story not linked back to a PRD.

### Check 5: Template compliance

- PRD: verify all 11 required sections are present (Overview, Goals, Non-Goals, Users, Requirements, UI/UX, Technical, Risks, Dependencies, Metrics, Open Questions).
- Stories: verify Given/When/Then AC format.
- Research docs: verify at least one source is cited.

### Check 6: Language consistency

- All docs must use `spec_lang` throughout.
- Flag any document that mixes languages within a single file.

---

## Step 3: Report

Display validation results. Example format (adapt to `lang`):

**English:**
```
Cross-document Validation Report

  ✅ Cross-references: 12/12 valid
  ⚠️  Naming: 1 file doesn't match pattern (STORY-003 missing prefix)
  ❌  TBDs: 2 orphan TBDs (PRD section 5.2, Story S-003)
  ✅ Traceability: 8/8 requirements → stories
  ✅ Template compliance: all sections present
  ✅ Language: consistent

  Overall: ⚠️  1 warning, 1 error — needs fixes
```

**Vietnamese:**
```
Báo cáo Kiểm Tra Tài Liệu

  ✅ Cross-references: 12/12 hợp lệ
  ⚠️  Naming: 1 file không đúng pattern (STORY-003 thiếu prefix)
  ❌  TBDs: 2 TBD không có owner (PRD mục 5.2, Story S-003)
  ✅ Traceability: 8/8 requirement → story
  ✅ Template: đầy đủ các section
  ✅ Ngôn ngữ: nhất quán

  Tổng hợp: ⚠️  1 cảnh báo, 1 lỗi — cần sửa trước khi giao
```

Then use AskUserQuestion (in `lang`):

**If lang = en:**
```json
{
  "questions": [{
    "question": "Validation done. What would you like to do?",
    "header": "Next step",
    "multiSelect": false,
    "options": [
      {"label": "Fix issues", "description": "Re-run /compass:run for the affected Colleagues"},
      {"label": "Proceed to delivery", "description": "Accept as-is and push to Jira/Confluence"},
      {"label": "Review manually", "description": "Stop here — I'll fix things myself"}
    ]
  }]
}
```

**If lang = vi:**
```json
{
  "questions": [{
    "question": "Đã kiểm tra xong. Bạn muốn làm gì tiếp?",
    "header": "Bước tiếp theo",
    "multiSelect": false,
    "options": [
      {"label": "Sửa lỗi", "description": "Chạy lại /compass:run cho Colleague bị lỗi"},
      {"label": "Giao hàng luôn", "description": "Chấp nhận kết quả và đẩy lên Jira/Confluence"},
      {"label": "Xem xét thủ công", "description": "Dừng ở đây — tôi sẽ tự sửa"}
    ]
  }]
}
```

- **Fix issues** → tell user which Colleague(s) to re-run (e.g. `/compass:run prd` or `/compass:run story`). Stop here.
- **Review manually** → stop, no action.
- **Proceed to delivery** → continue to Step 3b.

---

## Step 3b: PRD taste validation (blocking gate before delivery)

Before any external push (Jira/Confluence) in Step 4, run the v1.0 PRD taste validator on every PRD artifact collected in Step 1. This gate enforces the deterministic taste rules defined in `core/shared/SCHEMAS-v1.md` section 4: **R-FLOW** (User Flows must be ordered numeric lists) and **R-XREF** (every `[LINK-…]`, `[EPIC-…]`, `[REQ-…]` reference must resolve).

**For each PRD path** in the session outputs and in any active pipeline `artifacts` of type `prd`:

```bash
compass-cli validate prd <prd_path>
```

The CLI exits `0` on pass; exits `1` and emits JSON `{ "ok": false, "violations": [{ "rule", "line", "message" }] }` on failure.

**Aggregate violations** across all PRDs. If any PRD fails:

1. **Present violations grouped by rule** (`R-FLOW` first, then `R-XREF`). For each violation show:
   - PRD file path
   - Line number (1-based)
   - The offending token or section name + line range
   - The rule ID

   Example render (adapt to `lang`):
   ```
   PRD taste violations — delivery BLOCKED

     R-FLOW (2 violations)
       prd/SV-2026-04-13-bulk-upload.md:42
         Section 'User Flows > Sign-up' lines 42-45: expected ordered list, found unordered bullets.
       prd/SV-2026-04-13-bulk-upload.md:58
         Section 'User Flows > Error path' lines 58-60: expected ordered list, found prose paragraph.

     R-XREF (1 violation)
       prd/SV-2026-04-13-bulk-upload.md:108
         Dangling reference [REQ-17]: no matching anchor in file; not found under PRDs/, Stories/, Backlog/, epics/.
   ```

2. **Block delivery.** Use AskUserQuestion (in `lang`) to let the user choose:

   **English:**
   ```json
   {
     "questions": [{
       "question": "PRD taste violations detected. Delivery is blocked. What next?",
       "header": "Taste gate",
       "multiSelect": false,
       "options": [
         {"label": "Pause for author fix", "description": "Stop here so the PRD author can fix the violations, then re-run /compass:check"},
         {"label": "Skip this PRD", "description": "Exclude the failing PRD(s) from this delivery and continue with the rest"},
         {"label": "Abort delivery", "description": "Cancel the entire delivery for this session"}
       ]
     }]
   }
   ```

   **Vietnamese:**
   ```json
   {
     "questions": [{
       "question": "Phát hiện vi phạm taste rules của PRD. Giao hàng bị chặn. Bạn muốn làm gì?",
       "header": "Taste gate",
       "multiSelect": false,
       "options": [
         {"label": "Dừng để tác giả sửa", "description": "Dừng ở đây để tác giả PRD sửa vi phạm, sau đó chạy lại /compass:check"},
         {"label": "Bỏ qua PRD này", "description": "Loại PRD bị lỗi khỏi lần giao này và tiếp tục với phần còn lại"},
         {"label": "Huỷ giao hàng", "description": "Huỷ toàn bộ phần giao hàng của phiên này"}
       ]
     }]
   }
   ```

3. **Route by user choice:**
   - **Pause for author fix** → stop the workflow. Do NOT enter Step 4. Tell the user to fix the listed violations and re-run `/compass:check`.
   - **Skip this PRD** → drop the failing PRD(s) from the delivery set. Proceed to Step 4 with the remaining artifacts only. Record the skipped PRDs in the final summary (Step 5) under a "Skipped (taste violations)" bucket.
   - **Abort delivery** → stop. No Jira/Confluence push. If `pipeline_active = true`, leave `pipeline.json` as-is (do NOT mark completed).

**If all PRDs PASS** (`validate prd` exits 0 for every path) → proceed to Step 4. No prompt needed; the gate is transparent on success.

**Edge case — no PRD artifacts in the session**: skip Step 3b silently and proceed to Step 4.

---

## Step 4: Delivery (optional)

### Jira push

Check `integrations_override.jira` or `~/.compass/core/integrations/jira.md` for connection status.

If Jira configured:
- Propose: create Epic from PRD title, create Stories as Jira tickets under Epic, link dependencies.
- Use AskUserQuestion to confirm before pushing (show what will be created).
- On confirm → push via Jira MCP tools.
- On cancel → skip silently.

If Jira not configured → show (in `lang`):
- en: `"Jira not connected. Run /compass:setup jira to configure it."`
- vi: `"Jira chưa được kết nối. Chạy /compass:setup jira để cấu hình."`

### Confluence push

Check `integrations_override.confluence` status.

If Confluence configured:
- Publish PRD to team wiki space.
- Link Confluence page URL back into the Jira Epic description.
- Use AskUserQuestion to confirm space/page title before publishing.

If Confluence not configured → show (in `lang`):
- en: `"Confluence not connected. Run /compass:setup confluence to configure it."`
- vi: `"Confluence chưa kết nối. Chạy /compass:setup confluence để cấu hình."`

---

## Step 4a: Update project memory (after successful delivery)

Immediately after a successful Jira / Confluence push (or immediately after Step 4 if no integrations ran but the user chose "Proceed to delivery"), record the delivered artifacts and any newly discovered conventions into the project's durable memory via the CLI.

The memory schema is defined in `core/shared/SCHEMAS-v1.md` section 2 (`project-memory.json` v1.0). Use the CLI patch command so the CLI handles FIFO rotation, timestamp updates, and de-duplication:

```bash
compass-cli memory update "$PROJECT_ROOT" --patch '<json>'
```

Where `<json>` is a JSON object describing the patch to apply. Construct the patch from the current session:

- `sessions[]` — append an entry for the just-closed session with:
  - `session_id` (the session slug from Step 1)
  - `slug`
  - `finished_at` (current ISO-8601 UTC timestamp)
  - `deliverables` — the list of artifact paths that were actually delivered (PRDs that passed the Step 3b taste gate plus any Stories / research docs pushed to Jira/Confluence; exclude anything dropped under "Skip this PRD")
  - `decisions` — any decisions surfaced during validation or delivery (e.g. "Skipped PRD X due to unresolved `[REQ-…]` refs")
  - `discovered_conventions` — new conventions observed across the validated outputs (e.g. naming pattern variants the PO confirmed, a new glossary term adopted by the team). Each entry has `area`, `convention`, `source_session`.
  - `resolved_ambiguities` — any open questions the delivery answered in passing.

Example patch (shape only — derive real content from the current session):

```json
{
  "sessions_append": {
    "session_id": "<slug>",
    "slug": "<slug>",
    "finished_at": "<ISO-8601>",
    "deliverables": ["prd/SV-2026-04-14-feature-x.md", "epics/SV-EPIC-07/user-stories/SV-STORY-001-…md"],
    "decisions": [],
    "discovered_conventions": [],
    "resolved_ambiguities": []
  }
}
```

**Error handling**:
- If the CLI exits non-zero (e.g. memory file locked, patch rejected), show a one-line warning in `lang` but DO NOT fail the delivery — the artifacts are already in Jira/Confluence. Log the patch JSON so the user can re-apply manually.
- If `project-memory.json` does not exist yet, the CLI creates it with `memory_version: "1.0"` and seeds `project_prefix` from `$PROJECT_ROOT/.compass/.state/config.json`.

**If delivery was aborted or all PRDs were skipped in Step 3b**: skip Step 4a entirely — nothing was delivered, so there is nothing to record.

---

## Step 4b: Close pipeline (if active)

**This step runs only when `pipeline_active = true`.**

After validation is complete (Step 3) and the PO chose "Proceed to delivery" or "Review manually" (not "Fix issues"):

1. Update `pipeline.json` — set `"status": "completed"`:
   ```json
   { "status": "completed", "completed_at": "<ISO 8601 timestamp>" }
   ```
   Merge this with the existing `pipeline.json` content — do NOT overwrite the full file.

2. Show a pipeline summary (in `lang`):

**English:**
```
Pipeline closed: <pipeline title>

  Session: .compass/.state/sessions/<slug>/
  Status:  completed

  Artifacts in this pipeline:
    <type>   <path>
    prd      prd/SV-2026-04-13-feature-name.md
    story    epics/SV-EPIC-01/user-stories/SV-STORY-001-create-cmk.md
    story    epics/SV-EPIC-01/user-stories/SV-STORY-002-rotate-cmk.md
    research research/SV-RESEARCH-market-analysis-2026-04-13.md
```

**Vietnamese:**
```
Đã đóng pipeline: <pipeline title>

  Phiên: .compass/.state/sessions/<slug>/
  Trạng thái: hoàn thành

  Các artifacts trong pipeline này:
    <loại>     <đường dẫn>
    prd        prd/SV-2026-04-13-feature-name.md
    story      epics/SV-EPIC-01/user-stories/SV-STORY-001-create-cmk.md
    story      epics/SV-EPIC-01/user-stories/SV-STORY-002-rotate-cmk.md
    research   research/SV-RESEARCH-market-analysis-2026-04-13.md
```

Derive the artifact list from the `artifacts` array in `pipeline.json` — show the actual paths, not placeholders.

If `pipeline_active = false` → skip this step entirely.

---

## Step 5: Final summary

Show a clean summary card (in `lang`):

**English:**
```
Check complete!

  Validation:   ⚠️  1 warning, 1 error
  Delivery:     ✅ Pushed to Jira + Confluence

  Files:
    ✅ research/TD-user-feedback-auth.md
    ✅ prd/TD-2026-04-12-auth-system.md
    ✅ epics/TD-EPIC-05/user-stories/TD-STORY-001-login.md
    ⚠️  epics/TD-EPIC-05/user-stories/STORY-003-signup.md  ← naming issue
```

**Vietnamese:**
```
Kiểm tra hoàn tất!

  Kết quả:      ⚠️  1 cảnh báo, 1 lỗi
  Giao hàng:    ✅ Đã đẩy lên Jira + Confluence

  Tài liệu:
    ✅ research/TD-user-feedback-auth.md
    ✅ prd/TD-2026-04-12-auth-system.md
    ✅ epics/TD-EPIC-05/user-stories/TD-STORY-001-login.md
    ⚠️  epics/TD-EPIC-05/user-stories/STORY-003-signup.md  ← lỗi naming
```

---

## Edge cases

- **No output files found** → error, suggest `/compass:run` first.
- **Jira configured but no connection** → skip Jira push with a one-line warning. Don't block Confluence.
- **Confluence configured but no connection** → skip with warning. Don't block other deliveries.
- **All checks pass** → auto-suggest delivery step; skip the "Fix issues" option in AskUserQuestion.
- **Mixed validation results** → let PO decide per-check; list failed checks clearly in the question description.
- **Session has partial outputs** → validate what exists; note missing files in the report.
- **spec_lang = bilingual** → check language consistency in both language versions of each doc.
- **No integrations configured** → skip Step 4 entirely after noting delivery was skipped.
