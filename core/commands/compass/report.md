---
name: compass:report
description: Generate a quarterly report — domain scope (all products in the domain) or product scope (current project). Aggregates release notes, epics, cross-domain dependencies; asks PO for commentary. Uses shared/templates/quarterly-report-template.md via resolver.
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

Output path must match CI naming pattern `reports/[DOMAIN_OR_PREFIX]-Q[N]-[YYYY]-report.md`. Frontmatter fields must include: title, type, domain, quarter, year, po_lead, status — enforced by `shared/ci/validate_frontmatter.py`. Template is resolved via `core/shared/template-resolver.md` — shared/templates/quarterly-report-template.md wins if present; bundled fallback otherwise (currently none for this type — warn + free-form).
