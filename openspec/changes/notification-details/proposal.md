# Proposal

## Why

Notifications currently discard failure context, and template validation publishes only a problem count. Operators need a readable explanation and copyable diagnostics.

## What Changes

- Open shared details from every toast and history entry, with keyboard controls and themed dialogs.
- Preserve structured failure reasons, root-first causes, identifiers and available technical diagnostics in durable history.
- Carry each parameter/file validation problem from its source into the notification.

## Capabilities

### New Capabilities

- `notification-details`: Accessible notification inspection and complete failure context.

### Modified Capabilities

None.

## Impact

Frontend notification delivery/history, native error projection, template validation, focused unit stories, GUI and product documentation. Reuse canonical protobuf contracts and startup consequence presentation. Commit only; check-only proof, no push or full CI simulation, as explicitly requested.
