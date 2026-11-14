# Canonical Build Specification

## Purpose
Define canonical parallel native compilation, preserved local extended coverage and accountable pipeline build costs.

## Requirements

### Requirement: Canonical Native Configuration

Ordinary Product and Tests SHALL use canonical configurations per platform and profile. Independent feature sets are permitted; compatible compiler outputs MAY be reused without widening the shipped feature graph. Necessary fault-injection instrumentation SHALL remain local extended coverage and SHALL NOT enter provider CI or shipping binaries. Local installers SHALL NOT build tests by default.

#### Scenario: Ordinary Review

- **WHEN** an ordinary review requests Product and Tests
- **THEN** separate parallel Product and Tests jobs per platform produce their exports, and downstream Test and Package consume required exports without rebuilding

### Requirement: Explicit Clippy Ownership

Build, test, bench, fixture and focused compilation tasks SHALL NOT run Clippy implicitly. Clippy SHALL run with warnings denied only through `lint:rust`, included by `check:rust` and gated by CI Check / Rust; an explicit focused `clippy` operation MAY invoke it deliberately.

#### Scenario: Lint the Canonical Configuration

- **WHEN** Rust checking selects the canonical configuration
- **THEN** the lint task runs Clippy with warnings denied
- **AND** ordinary compilation does not run Clippy

### Requirement: Pipeline Build Cost

Warm-cache provider pipelines SHALL finish within 45 minutes, targeting 30 minutes. Tests Build SHALL take no longer than Product Build for the same candidate, native platform and profile. Relocated fixture or starter work SHALL remain visible in total critical-path accounting. A breach SHALL trigger phase-level investigation of compilation, feature selection and test structure; required coverage SHALL NOT be removed silently.

#### Scenario: Exceed the Build Budget

- **WHEN** a measured candidate exceeds the pipeline ceiling or Tests Build exceeds Product Build
- **THEN** the responsible phases and configurations are identified and redundant work is repaired
- **AND** significant architecture or feature changes require a human decision
