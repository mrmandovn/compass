# Shared Module: UX Rules

This module defines the universal UX rules for all Compass workflows. Every workflow that references this module MUST follow all rules throughout its entire execution — from first user interaction to final confirmation.

---

> **UX RULES — READ THIS FIRST**
>
> 1. **NEVER show internal step names to the user.** "Step 0", "Phase A", "B1", "C3" are for YOU (the LLM) to follow. The user sees a clean, friendly wizard — not a technical document.
> 1b. **NEVER synthesize a menu from bash/CLI command blocks.** When a workflow contains bash blocks with commands like `compass-cli project add/use/list`, those are for YOU to EXECUTE conditionally (per the workflow's Steps), not options to offer the user. Present choices ONLY via `AskUserQuestion` calls that the workflow explicitly defines with human-readable labels — never paraphrase shell commands into a menu.
> 2. **NEVER echo the workflow file content** or quote from it in your output.
> 3. **Use AskUserQuestion for EVERY choice.** Don't print bullet lists and ask the user to type "a" or "b". Use the interactive picker.
> 4. **Be warm and concise.** One short sentence to introduce each question. No walls of text.
> 5. **Language**: ALL user-facing chat text MUST be in `lang` from `$PROJECT_ROOT/.compass/.state/config.json` (resolved per `core/shared/resolve-project.md`). Artifact/file content uses `spec_lang`. They can differ (e.g. chat in Vietnamese, PRD in English).
> 6. **NEVER use "Open text" or empty options.** Every AskUserQuestion MUST have ≥2 meaningful, context-aware suggestions. Scan the project (existing PRDs, stories, research, config) to generate smart defaults. The built-in "Type your own answer" already handles free input — your job is to SUGGEST, not to ask blank questions.
> 7. **AskUserQuestion format**: when this workflow says "Use AskUserQuestion", call the tool with this EXACT JSON structure. Do NOT invent your own format:
> ```json
> {"questions": [{"question": "...", "header": "...", "multiSelect": false, "options": [{"label": "...", "description": "..."}, ...]}]}
> ```
> Every call MUST have: `questions` (array), each with `question` (string), `header` (string), `multiSelect` (boolean), `options` (array of `{label, description}` objects). Missing any of these fields will cause "invalid arguments" errors.

---

## Language enforcement

After resolving the active project per `core/shared/resolve-project.md`, extract `lang` from `$CONFIG`:
- All subsequent user-facing chat text MUST be in `lang`. No drift.
- If `lang` is missing from config, default to `en`.
- Artifact file content follows `spec_lang` (may differ from `lang`).

## Execution rules

**MANDATORY — every rule below must be followed exactly.**

1. **Read the workflow file FIRST.** Before doing anything, read the workflow file from the execution_context. Do NOT interpret the command name and act on your own — the workflow file contains the actual instructions.

2. **Do NOT delegate by default.** Run everything in the main conversation so output is visible to the user inline. Only spawn subagents/subtasks when the workflow step explicitly instructs delegation (e.g. `/compass:run` stage execution, `/compass:story` Step 4b parallel breakdown, `/compass:research` Step 4 parallel agents). When delegating, also emit progress in the main chat — see rule 8.

3. **Batch questions within one section.** Claude Code's `AskUserQuestion` natively accepts 1–4 questions per call. When a workflow section has multiple independent questions, send them together in a SINGLE call using the `questions` array. Do NOT make sequential single-question calls inside the same section. Across sections, still walk section by section.

4. **1 file = 1 document.** When creating multiple documents, create a SEPARATE file for EACH. NEVER combine multiple stories/PRDs/epics into one file.

5. **Skip only what the workflow explicitly marks skippable.** Execute every step in order. A step MAY auto-fill from context (e.g. AUTOFILL hints in Step 0c of prd.md) or SKIP based on a branching variable (e.g. `PRD_TYPE = enhancement` skips Section C/F). Outside of those explicit branches, do not invent shortcuts.

6. **Template-first output.** When generating documents, fill the template section by section. Do NOT generate the entire document in one shot.

7. **Confirm before proceeding.** After each major section, ask: "Continue?" — unless the workflow specifies otherwise (some workflows confirm once at the end, not per section).

8. **Emit progress for long phases.** When entering an execution phase likely to take >30 seconds or with >2 discrete steps, apply the progress pattern from `core/shared/progress.md` — print a plan block, tick on completion, summary at the end. Never leave the user waiting in silence.

9. **Artifact language must be consistent — no mixed Eng/Vie.** When generating artifacts (PRD, story, epic, release note, report), the ENTIRE document must be in `spec_lang`. Templates (from `$SHARED_ROOT/templates/` or bundled) are structural skeletons — their headings, labels, and keywords are illustrative. When `spec_lang ≠ en`, translate ALL structural elements: section headings, table headers, AC keywords, placeholders. No single document should mix English headings with Vietnamese prose or vice versa. Technical terms (API names, product names, acronyms) may stay in their original language.

## Additional rules for specific workflows

Some workflows have workflow-specific UX rules that extend (not override) these base rules. Those are defined inline in the workflow file, directly below the shared module reference.
