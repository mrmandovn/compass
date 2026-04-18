# Worker Rules

Rules that every Compass worker subagent MUST follow when implementing a task.
These rules are non-negotiable unless explicitly overridden by project config.

> **Customization:** Copy this file to `.compass/worker-rules.md` in your project
> and modify as needed. The project-level file takes priority over this default.

---

## 1. Scope Control

- **Only modify files listed in `task.files`.** If you discover a file that also needs changes, report it — do NOT modify it yourself.
- **No refactoring outside scope.** Do not "improve" surrounding code, rename variables in untouched functions, or clean up imports you didn't add.
- **No new dependencies** unless the DESIGN-SPEC explicitly requires them. If you believe a dependency is needed, report it as a blocker.
- **Do not delete or modify existing tests** unless the task explicitly covers test changes. Adding new tests is fine; breaking existing ones is not.
- **No feature creep.** Implement exactly what the task describes. No "while I'm here" additions.

---

## 2. Code Quality

- **Match the project's existing style.** Indentation, naming conventions (camelCase vs snake_case), quote style, bracket placement — follow what's already there.
- **Do not add comments, docstrings, or type annotations** to code you did not write or change. Only add comments where the logic is not self-evident in code you authored.
- **No over-engineering.** No premature abstractions, no helper utilities for one-time operations, no design-for-the-future patterns. Three similar lines > a premature abstraction.
- **No unnecessary error handling.** Do not add validation, fallbacks, or try/catch for scenarios that cannot happen according to the spec. Trust internal code and framework guarantees. Only validate at system boundaries (user input, external APIs).
- **No backward-compatibility shims.** No renaming unused `_vars`, no re-exporting removed types, no `// removed` comments. If something is removed, remove it completely.

---

## 3. Security

- **Never hardcode secrets, API keys, tokens, or credentials.** Use environment variables or config files.
- **Do not introduce OWASP Top 10 vulnerabilities:** no SQL injection, XSS, command injection, path traversal, or insecure deserialization.
- **Sanitize at system boundaries.** Validate and sanitize user input, external API responses, and file paths at entry points.
- **If you notice existing insecure code** in files you're modifying, fix it only if it's within your task scope. Otherwise, report it.

---

## 4. Git Discipline

- **Do not commit** unless the orchestrator tells you to. The orchestrator handles commits via `/compass:commit`.
- **Do not commit:** `.env` files, credentials, large binaries, IDE config, or OS-generated files.
- **Do not amend, rebase, or force-push** existing commits.

---

## 5. Acceptance

- **Read all `context_pointers` before writing any code.** Understand the existing code first.
- **Run the acceptance command** before reporting done. Do not report success if acceptance fails.
- **Max 3 retry attempts** if acceptance fails:
  1. Attempt 1 — fix based on error output
  2. Attempt 2 — re-read context, look for missed patterns
  3. Attempt 3 — try alternative approach
- **If all 3 attempts fail:** stop, report the failure with full error details (command, stdout, stderr). Do NOT keep retrying.

---

## 6. Context Hygiene

- **Read only what you need.** Start with `context_pointers`, then `task.files`. Do not explore the entire codebase.
- **Do not read files unrelated to the task.** Every file read consumes context window — keep it focused.
- **If you need information not in your context:** report it as a blocker rather than guessing.

---

## 7. GitNexus — Code Intelligence

If the orchestrator tells you GitNexus is available (`GITNEXUS_AVAILABLE`), use it to understand code before modifying it.

### Before editing a symbol (function, class, method):

```
gitnexus_impact({target: "symbolName", direction: "upstream", repo: GITNEXUS_REPO})
```

Check the blast radius. If risk is HIGH or CRITICAL, report it to the orchestrator before proceeding.

### When you need to understand a symbol's callers/callees:

```
gitnexus_context({name: "symbolName", repo: GITNEXUS_REPO})
```

### Rules:

- **Impact before edit.** Run `gitnexus_impact` on every symbol you're about to modify.
- **HIGH/CRITICAL = report.** Do not proceed without orchestrator acknowledgment.
- **Fallback gracefully.** If GitNexus fails, fall back to Grep/Glob. Do not block.
- **GitNexus unavailable = skip.** Use Grep/Glob as usual.

---

## 8. Communication

- **Report, don't guess.** If something is ambiguous — report it as a blocker. Do not make assumptions.
- **On failure, provide evidence:** the exact command run, full stdout/stderr, and what you tried.
- **Be terse.** State what you did, what passed, what failed.

---

## 9. Anti-Fake Tests

- **MUST NOT duplicate production logic.** Call the production function; do not rewrite it inline.
- **MUST NOT mock the function under test.** Only mock collaborators and external dependencies.
- **Tests MUST exercise the actual production code path.** If the test passes whether the production function exists or not, the test is fake.
- **MUST NOT test only hardcoded values.** Assertions must follow a call to production code with realistic inputs.

---

## 10. Edge Case Checklist

- **Promise leak prevention.** Every subscription/timer/request MUST be cancelled on teardown.
- **Error handling in async code.** Every Promise/async MUST handle rejection explicitly.
- **Resource disposal.** File handles, DB connections, WebSockets MUST be closed in finally/teardown.
- **Race conditions.** Verify captured variables are still valid when async operations resolve.
