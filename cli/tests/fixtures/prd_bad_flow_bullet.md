# PRD: Sample Feature (Bulleted Flow)

## Overview
This PRD is a fixture that violates R-FLOW by writing the User Flows section
as unordered bullet points (`-`) instead of an ordered numeric list.

## Goals
- Exercise the R-FLOW rule detector against unordered bullets.
- Keep every other section well-formed so only R-FLOW fires.

## Requirements
- The system must support a sign-in flow.
- The system must support a password-reset flow.

## User Flows

### Sign-in

- User opens the app.
- User enters email and password.
- System validates credentials against the identity store.
- On success, system issues a session token and redirects to the dashboard.
- On failure, system shows an inline error and increments the rate-limit counter.

### Password reset

- User clicks "Forgot password" on the sign-in screen.
- User enters their email address.
- System sends a reset link to the registered email.
- User clicks the link and enters a new password.
- System persists the new credential and invalidates all existing sessions.

## Success Metrics
- Sign-in success rate above 95 percent over any rolling 7-day window.

## Out of Scope
- Social login providers.
