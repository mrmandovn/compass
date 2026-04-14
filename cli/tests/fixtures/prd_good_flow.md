# PRD: Sample Feature

## Overview
This PRD describes a sample feature used as a fixture for PRD taste-rule tests.
It is a minimal, well-formed document and should pass both R-FLOW and R-XREF.

## Goals
- Provide a reproducible fixture for the R-FLOW rule.
- Keep the shape close to real PRDs written by the writer colleague.
- Avoid any dangling cross-references.

## Requirements
- The system must support a sign-in flow.
- The system must support a password-reset flow.
- The system must log every auth event.

## User Flows

### Sign-in

1. User opens the app.
2. User enters email and password.
3. System validates credentials against the identity store.
4. On success, system issues a session token and redirects to the dashboard.
5. On failure, system shows an inline error and increments the rate-limit counter.

### Password reset

1. User clicks "Forgot password" on the sign-in screen.
2. User enters their email address.
3. System sends a reset link to the registered email.
4. User clicks the link and enters a new password.
5. System persists the new credential and invalidates all existing sessions.

## Success Metrics
- Sign-in success rate above 95 percent over any rolling 7-day window.
- Password-reset completion rate above 60 percent of initiations.

## Out of Scope
- Social login providers.
- Hardware token support.
