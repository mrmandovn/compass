---
name: compass:report
description: Generate a period report — pick period type (quarter / half / multi-quarter / annual / custom date range) and scope (domain or current project). Aggregates release notes, epics, cross-domain dependencies; asks PO for commentary. Uses shared/templates/period-report-template.md via resolver.
allowed-tools:
  - Read
  - Write
  - Bash
  - Glob
  - AskUserQuestion
---

## Workflow

Read and execute the workflow at `~/.compass/core/workflows/report.md`.

## Instructions

- Follow the workflow Steps in order. If a Step says "Apply the shared snippet from `core/shared/<x>.md`", read that file and execute its logic inline — do not skip or paraphrase.
- Bash blocks in the workflow are commands for you to run; AskUserQuestion blocks are user choices. Never synthesize menus from bash/CLI command listings.
- Do not skip interactive wizard questions — always call AskUserQuestion where the workflow specifies, even if defaults look reasonable.

## Notes

Output path must match one of 5 CI naming patterns — quarter: `[PREFIX]-Q[N]-YYYY-report.md`, half: `[PREFIX]-H[1-2]-YYYY-report.md`, multi-quarter: `[PREFIX]-Q[N]Q[M](Q[K])-YYYY-report.md`, annual: `[PREFIX]-FYYYYY-report.md`, custom: `[PREFIX]-YYYYMMDD-YYYYMMDD-report.md`. Frontmatter must include: title, type=period-report, domain, period_type, period_label, period_start, period_end, po_lead, status — enforced by `shared/ci/validate_frontmatter.py`. Legacy `type: quarterly-report` files (with quarter+year fields) continue to validate under the old rule. Template is resolved via `core/shared/template-resolver.md` — shared/templates/period-report-template.md wins if present; bundled fallback otherwise (currently none for this type — warn + free-form).
