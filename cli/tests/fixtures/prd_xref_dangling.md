# PRD: Sample Feature (Xref Dangling)

## Overview
This PRD intentionally contains dangling cross-references [EPIC-999] and
[REQ-404]. Neither token resolves to a same-file anchor nor to any file
under PRDs/, Stories/, Backlog/, or epics/. Only the R-XREF rule should fire.

## Goals
- Exercise the R-XREF rule against unresolved tokens.
- Keep the User Flows section ordered so R-FLOW remains clean.

## Requirements
- The system must provide a sign-in flow (see [EPIC-999]).
- The system must provide a reset flow (see [REQ-404]).

## User Flows

### Sign-in

1. User opens the app.
2. User enters email and password.
3. System validates credentials.
4. On success, system issues a session token.
5. On failure, system shows an inline error.

### Password reset

1. User taps "Forgot password".
2. User enters their email.
3. System sends a reset link.
4. User sets a new password.
5. System invalidates existing sessions.

## Success Metrics
- Sign-in success rate above 95 percent rolling 7 days.

## Out of Scope
- Social login providers.
