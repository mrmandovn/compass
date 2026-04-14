# PRD: Sample Feature (Xref Valid)

## Overview
This PRD contains cross-references [LINK-API-SPEC], [EPIC-AUTH], and [REQ-SIGNIN]
that all resolve to same-file anchors. The User Flows section is well-formed
ordered lists so only the R-XREF rule would fire (and it should pass here).

## Goals
- Verify that R-XREF resolves tokens to same-file heading slugs.
- Keep flow section ordered so R-FLOW remains clean.

## Requirements
- Covered by [REQ-SIGNIN] and [REQ-RESET] below.

## LINK-API-SPEC
The feature relies on the internal authentication API, version 2.
See the service contract owned by the platform team for payload shape.

## EPIC-AUTH
This feature is part of the broader authentication epic covering
sign-in, sign-out, and credential recovery.

## REQ-SIGNIN
The system must authenticate a registered user with email and password,
issue a session token on success, and log the event to the audit stream.

## REQ-RESET
The system must allow a registered user to reset their password via an
emailed reset link with a time-limited token.

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
