# Shared: Progress Emission

**Purpose**: when a workflow enters an execution phase (writing, analysis, multi-step processing), it MUST emit a visible progress block in the main chat so the user sees what's happening without waiting in silence.

**Applies to**: any workflow that, after gathering input, enters a phase lasting >30 seconds or with >2 discrete steps.

---

## Pattern 1 — Inline execution (no delegation)

Used when the main agent does the work directly (e.g. `/compass:prd` writing all 6 sections).

### Step A — Emit the plan (before starting)

Print a plan block to main chat:

```
📋 <Artifact>: <title>
   Type: <type>   Sections: <N>
   Expected: <rough estimate, e.g. 2-4 min>

   ⏸  Step 1 — <label>
   ⏸  Step 2 — <label>
   ⏸  ...
```

- Use `⏸` for pending, `🔄` for in-progress, `✓` for done, `✗` for failed.
- Keep labels short (3-6 words).
- Don't overpromise timing — give a range, not a single number.
- Do not print a fake ETA countdown. Only note elapsed time when a step completes.

### Step B — Tick on each completion

After finishing each step, re-emit the block (or just the updated line) with:

```
✓ Step N — <label> (<elapsed>s)
🔄 Step N+1 — <label>
```

### Step C — Final summary

When done, print once:

```
✅ Done: <artifact path>
   Total: <elapsed>   Sections: <N>   Next: <suggested command>
```

### Example — `/compass:prd` inline

```
📋 PRD: E2EE Storage
   Type: new-feature   Sections: 6
   Expected: 3-5 min

   ⏸  Section A — Identity
   ⏸  Section B — Problem
   ⏸  Section C — Users
   ⏸  Section D — Goals
   ⏸  Section E — Metrics
   ⏸  Section F — Flow
```

After writing Section A:

```
✓ Section A — Identity (4s)
🔄 Section B — Problem
```

---

## Pattern 2 — Delegated execution (Task / sub-agent)

Used when the workflow spawns sub-agents (e.g. `/compass:run` stage-by-stage, `/compass:story` 4b parallel, `/compass:research` parallel codebase agents).

### Step A — Emit the delegation plan

Use a **markdown bullet list** so each colleague renders on its own line in Claude Code and OpenCode. Do NOT rely on leading-space indentation — renderers collapse it.

```
🚀 Delegating to <N> colleagues (Stage <W> of <total>, expected <range>):

- 🔄 **Product Writer** — drafting PRD sections B-F
- 🔄 **Research Aggregator** — scanning competitor docs
- ⏸ **Story Breaker** — waiting on PRD
```

- Bold the colleague name so it stands out.
- Describe what they're doing in 3-6 words after the em dash.
- Running: `🔄`. Waiting on dependency: `⏸`.
- Duration range (`~45-90s`, `3-5 min`) goes in the header line, not a separate block.

### Step B — Emit colleague-by-colleague completion

Same bullet list, tick each line as colleagues finish. Do NOT reprint the whole list — update inline.

```
- ✓ **Product Writer** — PRD drafted (32s)
- ✓ **Research Aggregator** — 5 sources aggregated (28s)
- 🔄 **Story Breaker** — breaking into 6 stories
```

### Step C — Stage transition (for `/compass:run`)

```
─── Stage 1 complete (58s, 2 artifacts) ───

🚀 Starting Stage 2 of 3:

- 🔄 **Story Breaker**
- 🔄 **Consistency Reviewer**
```

### Step D — Final summary

```
✅ All stages complete
   Artifacts: <N>   Total: <elapsed>   Session: <path>
```

---

## Rules

| Rule | Detail |
|------|--------|
| **Emit before starting** | Never start a long phase silently. Plan block first. |
| **Tick on every completion** | Update after each step/colleague — don't batch updates. |
| **Use real colleague names** | From `core/colleagues/manifest.json` — not generic "Agent 1". |
| **No fake ETA** | Ranges only ("3-5 min"), not countdowns or precise predictions. |
| **Main chat only** | Progress MUST appear in the main chat. Do not hide inside sub-agent output. |
| **Terse labels** | 3-6 words per step. No paragraphs. |
| **Failures visible** | If a step fails, use `✗` and state the error in one line. |

---

## How to invoke from a workflow

Add near the start of the execution phase:

```
Apply the progress emission pattern from `core/shared/progress.md`.
Pattern: <1 for inline | 2 for delegated>
Steps to track: <list the step labels>
```

The model then emits the plan block, ticks each step, and prints the final summary without further instruction.
