# Product Handoff Specification

## Purpose
Preserve complete product behavior, branding and native delivery contracts when reconciling divergent handoff branches.

## Requirements

### Requirement: Preserve The Complete Product Handoff
The integration SHALL include the intended product work from both existing handoff branches and the native CI migration in one candidate, with explicit resolution of overlapping ownership.

#### Scenario: Divergent Input Branches
- **WHEN** the drop and specification-completion branches each contain changes absent from the other
- **THEN** the source inventory accounts for both and no product capability is omitted merely because the CI migration began from older dev
- **AND** contradictory public contracts are resolved before source closure

### Requirement: Ship The Intended Branding
The app SHALL use the reconciled current branding assets and sidebar presentation through one authoritative asset ownership chain.

#### Scenario: Native App And Package
- **WHEN** the integrated product builds and packages its GUI and runtime resources
- **THEN** manifests, generated exports, runtime reports and the visible app use the intended surface-aware artwork
- **AND** obsolete asset ownership is removed rather than kept as a competing fallback

### Requirement: Preserve Migration Contracts
The combined tree SHALL preserve the approved native CI, client naming, publication boundaries, package freshness and cross-platform correctness contracts.

#### Scenario: Conflicting Product And Tooling Edits
- **WHEN** a product branch overlaps a migration repair
- **THEN** explicit conflict resolution preserves the intended product behavior and approved migration contract
- **AND** generated root and starter workflows derive from the reviewed Hub source without provider protection mutations

### Requirement: Prove The Combined Candidate
Integration into dev SHALL require applicable full local proof and every required native provider check on the complete combined candidate.

#### Scenario: Older Baseline Passed
- **WHEN** earlier validation passed before missing product changes were integrated
- **THEN** those results remain baseline evidence and do not substitute for proof of the combined source
- **AND** the validated candidate is delivered through the existing protected PR

### Requirement: Isolate Explicit Desktop Storage
Desktop organization packs SHALL use a supplied nonempty `DOKIME_APPLICATION_ROOT` and otherwise retain their existing platform application-data location.

#### Scenario: Isolated Native Organization Journey
- **WHEN** the desktop starts with an explicit application root and imports an organization pack
- **THEN** pending and active packs persist beneath that root across restarts
- **AND** the ordinary user organization store remains untouched

### Requirement: Use Native Python Dependency Locking
Python Build and Package SHALL NOT enforce a custom wheel-byte hash comparison. Ordinary uv dependency locking and package version/source checks SHALL remain; no ZIP normalization or replacement custom integrity gate is required.

#### Scenario: Platform-Specific Wheel Metadata
- **WHEN** a built wheel has platform-dependent archive metadata
- **THEN** custom hash comparison does not reject it
- **AND** the delivered tutorial remains usable through ordinary uv dependency installation

## Workflow Follow-Up

- Required native provider checks, protected integration into dev and authorized release verification remain delivery obligations; local proof and specification retirement do not establish their completion.
- Benchmark timings and optimized-profile build-cost measurements are optional follow-up. No baseline speedup is claimed from unperformed measurements.
