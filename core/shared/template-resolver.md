# Shared: Resolve Template

**Purpose:** Find the best template file for a given artifact type. Silver Tiger `shared/templates/` is authoritative when available; Compass bundled templates are the fallback for standalone projects without shared/.

**Used by:** Any workflow that produces artifacts from a template (prd, story, epic, research, roadmap, release, sprint, feedback, prioritize, prototype).

---

## How to invoke

When a workflow needs a template, it calls this resolver with one argument: the **template name** (matching the filename in `shared/templates/` without the `.md` extension).

```
TEMPLATE_NAME="<name>"    # e.g. "prd-template", "user-story-template", "epic-template"
```

## Lookup order (stop at first match)

```bash
# 1. Silver Tiger shared/ (authoritative when present)
CANDIDATE_1="$SHARED_ROOT/templates/${TEMPLATE_NAME}.md"

# 2. Compass bundled fallback
CANDIDATE_2="$HOME/.compass/core/templates/${TEMPLATE_NAME}.md"

# Resolve
if [ -n "$SHARED_ROOT" ] && [ -f "$CANDIDATE_1" ]; then
  TEMPLATE_PATH="$CANDIDATE_1"
  TEMPLATE_SOURCE="shared"
elif [ -f "$CANDIDATE_2" ]; then
  TEMPLATE_PATH="$CANDIDATE_2"
  TEMPLATE_SOURCE="bundled"
else
  TEMPLATE_PATH=""
  TEMPLATE_SOURCE="none"
fi

echo "TEMPLATE_PATH=$TEMPLATE_PATH"
echo "TEMPLATE_SOURCE=$TEMPLATE_SOURCE"
```

## After resolution

**If `TEMPLATE_SOURCE=shared`:**
- Use the template as-is. Silver Tiger team maintains it — Compass trusts its structure and language.
- If `spec_lang` differs from the template's language, apply the language directive from `core/shared/ux-rules.md` Rule 9 (translate headings, labels, and prose entirely — no mixed language).

**If `TEMPLATE_SOURCE=bundled`:**
- Use the Compass bundled template. It may have a different section structure than shared/.
- Always apply Rule 9 language directive when `spec_lang ≠ en` (bundled templates are English-only).

**If `TEMPLATE_SOURCE=none`:**
- Print a warning: `⚠ No template found for "${TEMPLATE_NAME}". Proceeding without a template — output structure may vary.`
- The workflow continues with free-form generation (no template skeleton). This is acceptable for artifact types that don't have a template yet (e.g. ideate, feedback).

## Template name mapping

Workflows use these template names (matching `shared/templates/` filenames):

| Workflow | Template name | shared/ file | Bundled fallback |
|---|---|---|---|
| `/compass:prd` | `prd-template` | `prd-template.md` | `prd-template.md` |
| `/compass:story` | `user-story-template` | `user-story-template.md` | `user-story-template.md` |
| `/compass:epic` | `epic-template` | `epic-template.md` | _(none)_ |
| `/compass:release` | `release-note-template` | `release-note-template.md` | _(none)_ |
| `/compass:roadmap` | — | _(none)_ | _(none)_ — free-form |
| `/compass:sprint plan` (default) | — | _(none)_ | _(none)_ — free-form (plan is a lightweight list, no template) |
| `/compass:sprint review` | `sprint-review-template` | `sprint-review-template.md` | _(none)_ |
| `/compass:report` | `period-report-template` | `period-report-template.md` | _(none)_ |
| `/compass:research` | — | _(none)_ | _(none)_ — free-form |
| `/compass:ideate` | — | _(none)_ | _(none)_ — free-form |
| `/compass:feedback` | — | _(none)_ | _(none)_ — free-form |
| `/compass:prioritize` | — | _(none)_ | _(none)_ — free-form |
| `/compass:prototype` | — | _(none)_ | _(none)_ — free-form |

## Rules

- **shared/ always wins** when both shared/ and bundled exist for the same template name.
- **Never modify shared/ templates from Compass workflows.** They are read-only references maintained by the Silver Tiger PO team.
- **`$SHARED_ROOT`** comes from `core/shared/resolve-project.md` Step 0b-bis. If it was not resolved (empty), skip candidate 1.
- **Template names are stable.** If shared/ renames a template, update the mapping table above.
