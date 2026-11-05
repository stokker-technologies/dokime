# GUI Failure Presentation

## Why

GUI failures currently lose diagnostic context in message-only notices and follow inconsistent notification and console paths. Operators need a readable inline failure for unavailable data and complete, copyable diagnostics everywhere.

## What Changes

- Implement the owner's agreed design: extend `NotificationFailure` with title, identity and client stack; `failureOf(error, title)` is the sole leaf constructor.
- Add destructive ErrorCard with retry, collapsed technical details and complete-record Copy; keep PaneNotice for loading and empty states only.
- Route unavailable views to ErrorCard and console; failed actions and background failures to toast, history and console. Report each failure identity once.
- Preserve canonical backend decoding, redacted diagnostics, remediation actions, notification history and the shared details dialog. Delete superseded extraction helpers and migrate the whole GUI.
- Document routing and verify deterministic behavior, TypeScript checks and a failing desktop view when supported.

## Capabilities

### New Capabilities

- `gui-failure-presentation`: One normalized failure record and consistent presentation routing.

### Modified Capabilities

## Impact

Frontend failure models, notifications, inline views, action and background callers, GUI stories, GUI documentation and the README feedback claim. No backend contract, storage reset or compatibility migration.

## Agreement

The user supplied and approved the owner-locked design, scope, scenarios, proof and delivery in the task instruction. Other open changes are outside this inventory.
