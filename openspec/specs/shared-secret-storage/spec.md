# Shared Secret Storage Specification

## Purpose
Define host-owned shared secrets and authenticated, bounded transfer without exposing secret values.

## Requirements

### Requirement: Host-Owned Shared Secret Storage

The host SHALL provide freely managed stable named secrets shared through plugin lookup, while app authentication tokens and signing keys remain separate. Organization Packs SHALL contain neither secret values nor plugin secret binding declarations.

#### Scenario: Shared Lookup And Management

- **WHEN** multiple plugins look up a name that is added, replaced or deleted
- **THEN** future lookups see its current value or a clear missing-secret error, existing required-secret checks fail before device work where available, and settings show the owning host without exposing values or claiming exhaustive usage.

#### Scenario: Encrypted Transfer And Conflict Preview

- **WHEN** an operator exports selected secrets and imports them with explicit conflict choices
- **THEN** authenticated password encryption protects the transfer, wrong passwords/tampering/invalid or oversized input write nothing, and accepted replace/skip choices publish atomically with no raw values in logs, reports or evidence.

#### Scenario: Secure Backend Requirement

- **WHEN** an operator attempts a new shared-secret write or import without OS keyring storage
- **THEN** the host reports the secure-storage requirement and preserves existing LocalFile data for explicit secure adoption without silent deletion or rewriting.
