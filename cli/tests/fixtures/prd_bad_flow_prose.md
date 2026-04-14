# PRD: Sample Feature (Prose Flow)

## Overview
This PRD is a fixture that violates R-FLOW by writing the User Flows section
as prose paragraphs instead of an ordered numeric list.

## Goals
- Exercise the R-FLOW rule detector against prose.
- Keep every other section well-formed so only R-FLOW fires.

## Requirements
- The system must support a sign-in flow.
- The system must support a password-reset flow.

## User Flows

The sign-in flow begins when the user opens the application and is greeted
by the landing screen. They tap the sign-in button, enter their email and
password, and submit the form. The system then validates the credentials
against the identity store and, on success, issues a session token and
redirects the user to the dashboard.

The password-reset flow starts when a user taps the forgot-password link
on the sign-in screen. The system prompts for their email, sends a reset
message, and waits for the user to follow the embedded link. Once the user
sets a new password, the system persists it and invalidates all existing
sessions.

## Success Metrics
- Sign-in success rate above 95 percent over any rolling 7-day window.

## Out of Scope
- Social login providers.
