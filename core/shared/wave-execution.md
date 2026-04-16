# Shared: Wave Execution (Fresh-Context Sub-Agents)

**Purpose**: Execute a wave of implementation tasks by spawning a Claude Code sub-agent with **strictly scoped** context — zero carry-over from the main conversation or prior waves. This is the anti-context-pollution mechanism that makes iterative dev work practical on large codebases.

**Used by**: `/compass:build` (per wave), `/compass:fix` (single wave).

---

## The Pattern

```
┌─────────────────────────────────────────────────────────────┐
│ Main agent (workflow orchestrator)                          │
│  - Reads plan.json                                           │
│  - Loops waves sequentially                                  │
│  - For each wave: spawn fresh sub-agent, verify, commit      │
│  - Does NOT think about implementation details itself        │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│ Sub-agent (Claude Code Agent tool, general-purpose)          │
│  - Fresh context window (no prior messages visible)          │
│  - Prompt = ONLY the wave's scope (CONTEXT + spec sections   │
│    for this wave + task list + constraints)                  │
│  - Implements, runs tests, reports back                      │
│  - Does NOT commit (main agent handles git)                  │
└─────────────────────────────────────────────────────────────┘
```

**Why**: When main agent does implementation in its own context, (a) it sees everything from prior waves (patterns, file contents, decisions) → future waves accidentally reference out-of-scope code; (b) its context window fills up and later waves get truncated. Sub-agent isolation solves both.

---

## Spawn template

```
For each wave W in plan.waves (sequential, wave N+1 only starts after wave N done):

  ┌─ STEP A: Build context pack ─────────────────────────────────┐
  │ For each task T in W.tasks:                                  │
  │   compass-cli context pack "$SESSION_DIR" "$T.task_id"       │
  │   → appends to $CONTEXT_PACK (file content bundle)           │
  │ Dedupe files across tasks.                                   │
  └──────────────────────────────────────────────────────────────┘

  ┌─ STEP B: Build sub-agent prompt ─────────────────────────────┐
  │ PROMPT = <<END                                               │
  │ # Wave <W.wave_id> — <W.title>                                │
  │                                                              │
  │ You are implementing a wave of code changes. Read the        │
  │ context below, then implement ONLY the wave tasks listed.    │
  │                                                              │
  │ ## Strict scope rules                                        │
  │ - Files you may modify: <union of W.tasks[*].files_affected> │
  │ - Do NOT touch files outside this list                       │
  │ - Do NOT refactor unrelated code, even if you see quality    │
  │   issues — they are out of scope                              │
  │ - Do NOT add features beyond the task list                    │
  │ - Do NOT run git commit (main workflow handles commits)       │
  │                                                              │
  │ ## Context (decisions locked earlier — do not revisit)       │
  │ <verbatim content of CONTEXT.md>                              │
  │                                                              │
  │ ## Design Spec (relevant sections only)                      │
  │ <DESIGN-SPEC.md sections matching the REQ ids in            │
  │  W.tasks[*].covers[]>                                         │
  │                                                              │
  │ ## Test Spec (tests for this wave)                           │
  │ <TEST-SPEC.md sections matching the REQ ids above>            │
  │                                                              │
  │ ## Wave Tasks                                                │
  │ <for each T in W.tasks>                                      │
  │ ### $T.task_id — $T.name                                     │
  │ - Files affected: $T.files_affected                          │
  │ - Briefing: $T.briefing_notes                                │
  │ - Acceptance (run after implementing):                       │
  │     <T.acceptance.criteria joined with newlines>             │
  │ </for>                                                       │
  │                                                              │
  │ ## Execution steps                                           │
  │ 1. Read each file in files_affected to understand current    │
  │    state                                                     │
  │ 2. Implement tasks (edit/create files in files_affected)     │
  │ 3. Run ALL commands in acceptance.criteria for each task     │
  │ 4. If any test fails, read the error, make a targeted fix,   │
  │    re-run. Up to 2 retries.                                  │
  │ 5. Report back with this structure:                          │
  │    - status: "success" | "needs_human" | "partial"           │
  │    - files_changed: [{path, change_summary}]                 │
  │    - tests_run: [{command, exit_code, output_excerpt}]       │
  │    - retries_used: 0 | 1 | 2                                 │
  │    - notes: any ambiguity or decisions made                  │
  │ END                                                          │
  └──────────────────────────────────────────────────────────────┘

  ┌─ STEP C: Spawn via Claude Code Agent tool ──────────────────┐
  │ Agent(                                                       │
  │   description: "Implement wave <W.wave_id>",                 │
  │   subagent_type: "general-purpose",                          │
  │   prompt: PROMPT                                             │
  │ )                                                            │
  │ → Sub-agent runs in isolated context, returns single msg    │
  └──────────────────────────────────────────────────────────────┘

  ┌─ STEP D: Parse response ─────────────────────────────────────┐
  │ RESP = parse sub-agent summary                               │
  │ CASE RESP.status:                                            │
  │   "success" → proceed to Step E                              │
  │   "needs_human" → AskUserQuestion:                           │
  │       retry with dev input / skip wave / abort build         │
  │   "partial" → AskUserQuestion:                               │
  │       accept partial + mark remaining pending / retry all    │
  │       / abort                                                │
  └──────────────────────────────────────────────────────────────┘

  ┌─ STEP E: Main-agent verification ────────────────────────────┐
  │ Re-run all W.tasks[*].acceptance.criteria commands in bash.  │
  │ Capture exit codes.                                          │
  │ If pass → proceed to Step F.                                 │
  │ If fail → treat same as "needs_human" in Step D (Step D's    │
  │   retry loop applies).                                       │
  │                                                              │
  │ Sanity: sub-agent might report "success" even if a test      │
  │ fails (LLM confabulation). Main-agent re-run catches this.   │
  └──────────────────────────────────────────────────────────────┘

  ┌─ STEP F: Commit wave ────────────────────────────────────────┐
  │ AFFECTED=$(union of W.tasks[*].files_affected)               │
  │ git add $AFFECTED                                            │
  │                                                              │
  │ MSG = "<type>(<scope>): <W.title>"                            │
  │   type = task_type from state.json                           │
  │   scope = common prefix of AFFECTED (e.g. "auth", "api")     │
  │          or "" if no clear common prefix                     │
  │   W.title = plan.waves[W.wave_id].title                      │
  │                                                              │
  │ compass-cli git commit "$MSG"                                │
  │ → capture commit SHA                                         │
  └──────────────────────────────────────────────────────────────┘

  ┌─ STEP G: Persist state ──────────────────────────────────────┐
  │ compass-cli state update "$SESSION_DIR" '{                   │
  │   "waves": [{                                                │
  │     "wave_id": '$W.wave_id',                                 │
  │     "status": "done",                                         │
  │     "commit_sha": "'$SHA'",                                   │
  │     "test_results": {...},                                    │
  │     "retry_count": '$RETRIES',                                │
  │     "completed_at": "'$(date -u +%FT%TZ)'"                    │
  │   }]                                                          │
  │ }'                                                           │
  │ compass-cli progress save "$SESSION_DIR" "wave-$W.wave_id-done"  │
  └──────────────────────────────────────────────────────────────┘

  ┌─ STEP H: Review gate ────────────────────────────────────────┐
  │ AskUserQuestion:                                             │
  │   "Wave <W.wave_id> done. Continue?"                          │
  │   - Continue to wave N+1                                     │
  │   - Retry this wave (redo from Step A)                       │
  │   - Pause (save state, print resume hint, stop)              │
  │   - Abort build                                               │
  └──────────────────────────────────────────────────────────────┘

  Loop to next wave.
```

---

## Retry policy detail

When wave verification fails (Step D "needs_human" OR Step E local test fail):

| Attempt | Action |
|---|---|
| **1** (original) | Initial sub-agent run |
| **2** (auto-retry 1) | Re-spawn sub-agent with failure context added to prompt: `"Previous attempt failed. Error output: <log>. Fix the test failure."` |
| **3** (auto-retry 2) | Re-spawn with stronger guidance: `"Still failing. Consider reducing scope — implement one task at a time and re-test between each. Or read the failing test more carefully — the fix may require a different approach."` |
| **4+** (dev-input) | Stop auto-retrying. AskUserQuestion: (a) retry with human guidance (Type-your-own-answer for hint) (b) skip this wave (mark pending, continue to next) (c) abort build |

Total auto-retries: **2** before pausing. After dev input → 1 more retry attempt.

---

## Sub-agent prompt constraints (enforced)

The prompt template MUST always include:

1. **Files whitelist** — `files_affected` union. Sub-agent told to not touch anything else.
2. **No commit rule** — sub-agent must NOT run `git commit`. Main agent orchestrates all git.
3. **Reporting format** — structured status/files/tests/retries/notes.
4. **Test re-run instruction** — sub-agent reruns tests after each fix, not just first attempt.

---

## What's NOT in the sub-agent prompt

These are intentionally omitted to keep context small:

- DESIGN-SPEC sections unrelated to this wave's REQs
- TEST-SPEC sections unrelated
- Code files outside `files_affected`
- Prior waves' sub-agent summaries (main agent has them, sub-agent doesn't need)
- Main conversation history (zero by default since Agent tool gives fresh context)
- Other sessions' artifacts

If a task genuinely needs context from files outside its `files_affected`, those files should go in `context_pointers` (read-only reference) and the sub-agent reads them via bash/Read. `files_affected` = writable; `context_pointers` = readable only.

---

## Rules

| Rule | Detail |
|---|---|
| **One wave at a time** | Wave N+1 does not start until wave N is fully done (tests pass + commit + state updated). No parallel waves. |
| **Main agent orchestrates git** | Sub-agent prohibited from committing. |
| **Main agent re-runs tests** | Don't trust sub-agent's reported test results blindly — verify locally. |
| **Fresh context per spawn** | Always use Agent tool's new-context invocation, never continue from prior subagent. |
| **Prompt stays lean** | If the prompt exceeds ~50KB of text, the wave is probably too big — split into sub-waves. |
| **Retry on fail, not on partial success** | Partial success (some tasks done, others pending) asks dev, doesn't auto-retry the succeeded parts. |
