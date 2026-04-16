<!--
  Test Spec Template — 3 variants (code / ops / content)

  Matches DESIGN-SPEC category. Every REQ-xx from DESIGN-SPEC must map to at
  least one test case here (Covers field). Every test has a runnable command
  or concrete verification step.

  The strategy picker in /compass:spec Step 10 decides the emphasis per
  category (unit-heavy / integration / mixed for code; smoke / full / dry-run
  for ops; checklist for content).
-->

---
tests_version: "1.0"
spec_ref: "<component>-spec-v1.0"
component: "<MUST MATCH DESIGN-SPEC.md component field>"
category: "<code | ops | content>"
strategy: "<unit | integration | mixed | smoke | full | dry-run | checklist>"
language: "<same as DESIGN-SPEC>"
---

<!-- ================================================================ -->
<!-- CODE variant — include these sections for category=code           -->
<!-- ================================================================ -->

## Unit Tests

### Test: <descriptive_name_snake_case>
- **Covers**: [REQ-01]
- **Input**: <concrete values>
- **Setup**: <mocks/fixtures needed>
- **Expected**: <exact output>
- **Verify**: `<runnable command that returns exit 0 on pass>`

### Test: <descriptive_name>
- **Covers**: [REQ-02]
- ...

## Integration Tests

### Test: <descriptive_name>
- **Covers**: [REQ-03]
- **Setup**: <environment, fixtures, external services running>
- **Steps**:
  1. <step>
  2. <step>
- **Expected**: <outcome>
- **Verify**: `<runnable command>`

## Edge Cases

| Case | Input | Expected behavior | Covers |
|---|---|---|---|
| Empty input | `""` | Return default / throw ValidationError | REQ-01 |
| Boundary: max size | `10000` | Accept | REQ-02 |
| Boundary: over max | `10001` | Reject with 413 | REQ-02 |
| Concurrent access | 100 parallel requests | No deadlock, all succeed | REQ-03 |

## Test Data / Fixtures

<Mock data, factories, sample inputs.>

```json
{
  "sample_user": {"id": "u-001", "email": "test@example.com"},
  "sample_product": {...}
}
```

## Coverage Target

- Target: **≥ 80%** line coverage on changed files
- Critical paths: **100%**
- Measure: `npm run test -- --coverage` (or equivalent)

<!-- ================================================================ -->
<!-- OPS variant — include these sections for category=ops             -->
<!-- ================================================================ -->

## Pre-flight Checks

- [ ] Docker daemon running: `docker info`
- [ ] Required env file exists: `test -f .env`
- [ ] Required secrets configured: `[ -n "$API_KEY" ]`
- [ ] Dependency X deployed: `curl -f http://dep.internal/health`

## Integration Tests

<Called "Integration Tests" (not "Smoke Tests") so the validator accepts ops specs uniformly. In practice these are smoke + health checks.>

### Check: <descriptive_name>
- **Covers**: [REQ-01]
- **Command**: `<runnable command>`
- **Expected**: exit code 0, output contains `<expected string>`, or status = running
- **Timeout**: <max wait time — e.g. 30s>

### Check: <descriptive_name>
- **Covers**: [REQ-02]
- **Command**: `docker compose ps | grep worker | grep running`
- **Expected**: 1 line of output (worker running)
- **Timeout**: 60s

## Rollback Verification

### Rollback: <scenario name>
- **Trigger**: <what goes wrong that needs rollback>
- **Steps**: <rollback commands>
- **Verify**: `<command to confirm rollback worked>`

## Edge Cases

| Scenario | How to simulate | Expected behavior | Covers |
|---|---|---|---|
| Network failure mid-deploy | `docker network disconnect ...` | Graceful retry, no data loss | REQ-03 |
| Secret rotated during deploy | Rotate in parallel | Worker picks up new secret on next restart | REQ-04 |
| Disk full | Fill test volume | Deploy fails fast with clear error | REQ-05 |

## Content Quality Gates

- [ ] All scripts have `set -euo pipefail` or equivalent
- [ ] All commands idempotent (safe to re-run)
- [ ] Rollback tested in pre-flight environment
- [ ] No hard-coded secrets or credentials

<!-- ================================================================ -->
<!-- CONTENT variant — include these sections for category=content     -->
<!-- ================================================================ -->

## Deliverable Checklist

### [REQ-01] <deliverable name>
- [ ] File exists at expected path: `<path>`
- [ ] Covers required topics: <list>
- [ ] Follows style guide / formatting rules
- [ ] Examples included where applicable

### [REQ-02] <deliverable name>
- [ ] ...

## Review Criteria

| Criterion | How to verify | Covers |
|---|---|---|
| Accuracy | Cross-check claims with source docs / code | REQ-01 |
| Completeness | All sections from outline present and non-empty | REQ-02 |
| Audience-appropriate | No jargon for end-user docs; sufficient depth for dev docs | REQ-03 |
| Current | No outdated references (old versions, dead links, removed features) | REQ-04 |

## Content Quality Gates

- [ ] Spell-check passes (e.g. `cspell` or manual review)
- [ ] Internal links resolve (no 404s)
- [ ] External links resolve at time of writing
- [ ] Code blocks compile / run as documented
- [ ] Screenshots / diagrams current (if any)
- [ ] Table of contents matches actual structure
