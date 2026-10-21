# Shared Secret Storage

## Why

Split out of the 2026-10-21 feature drop by user scope decision; the drop ships items 1-49 and its consolidation.

## What Changes

- Host-owned shared named Secret Storage, live plugin lookup, secure management and authenticated encrypted transfer; preserve existing stored values and separate application authentication.

## Capabilities

### New Capabilities

- `shared-secret-storage`: host-owned shared named secrets with secure management and encrypted transfer.

### Modified Capabilities

- None.

## Impact

Model/credentials, domains/credentials, settings operations, runtime config/preferences/admission, server handlers, plugin/harness access, client credential persistence, GUI native authentication types and the new host Secret Storage card.
