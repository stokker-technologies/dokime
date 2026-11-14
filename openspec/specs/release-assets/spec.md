# release-assets Specification

## Purpose
Define native distribution and tool archives, explicit shipping plugin inputs, signed installer verification and credential-free release validation.

## Requirements

### Requirement: Preserve Required Native Validation

Release tooling SHALL preserve required Windows and Linux Check/Test coverage and the Release / Dry job on release reviews and in Nightly, and SHALL use credential-free release rehearsal.

#### Scenario: Inspect the Release Workflow

- **WHEN** product configuration is adapted
- **THEN** required native checks remain enabled, development release validation is retained and rehearsal does not upload artifacts
