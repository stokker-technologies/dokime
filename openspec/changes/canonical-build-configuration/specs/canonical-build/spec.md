# Canonical Build

## ADDED Requirements

### Requirement: Canonical Native Configuration

Ordinary Product and Tests SHALL use canonical configurations per platform and profile. Independent feature sets are permitted; compatible compiler outputs MAY be reused without widening the shipped feature graph. Clippy SHALL precede compilation with matching configuration. Necessary fault-injection instrumentation SHALL remain local extended coverage and SHALL NOT enter provider CI or shipping binaries. Local installers SHALL NOT build tests by default.

#### Scenario: Ordinary Review

- **WHEN** an ordinary review requests Product and Tests
- **THEN** separate parallel Product and Tests jobs per platform produce their exports, and downstream Test and Package consume required exports without rebuilding

### Requirement: Compatibility Coverage

All provider CI pipelines SHALL use only canonical configurations. The explicit local extended suite SHALL retain declared all-features, no-default-features and cross-target compatibility checks and necessary fault-injection cases.

#### Scenario: Explicit Extended Validation

- **WHEN** an operator explicitly requests local extended validation
- **THEN** all declared compatibility configurations and instrumented cases remain available
- **AND** ordinary CI and installer tasks do not select them implicitly


### Requirement: Pipeline Build Cost

Warm-cache provider pipelines SHALL finish within 45 minutes, targeting 30 minutes. Tests Build SHALL take no longer than Product Build for the same candidate, native platform and profile. Relocated fixture or starter work SHALL remain visible in total critical-path accounting. A breach SHALL trigger phase-level investigation of compilation, feature selection and test structure; required coverage SHALL NOT be removed silently.

#### Scenario: Exceed The Build Budget

- **WHEN** a measured candidate exceeds the pipeline ceiling or Tests Build exceeds Product Build
- **THEN** the responsible phases and configurations are identified and redundant work is repaired
- **AND** significant architecture or feature changes require a human decision
