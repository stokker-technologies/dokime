## MODIFIED Requirements

### Requirement: Share Native Compilation Ownership
Separate parallel native Product and Tests jobs per platform and profile SHALL produce their own exports. Repository tasks SHALL own canonical feature configurations, with matching Clippy and child-scoped warning denial. Independent Product and Tests feature sets are permitted while preserving shipping behavior and required ordinary test coverage. Check SHALL remain noncompiling; ordinary Test and Package SHALL consume separate existing exports without rebuilding them. Broad feature/target compatibility and fault-injection cases SHALL remain available only in an explicit local extended suite, never selected implicitly by provider CI or installers.

#### Scenario: Build A Native Candidate
- **WHEN** native CI builds the candidate
- **THEN** Product and canonical Tests producers run in parallel per platform
- **AND** artifact dependencies and any subsequent fixture preparation are explicit rather than hidden in consumers
- **AND** compatible compiler units may be reused without promising a single compilation of distinct targets
- **AND** consumers do not rebuild their required exports
- **AND** Tests Build takes no longer than Product Build, with warm-cache whole pipelines below 45 minutes and targeting 30; breaches require phase-level investigation without silently reducing coverage

#### Scenario: Build Shipping Artifacts Without Tests
- **WHEN** an ordinary Product or installer build runs
- **THEN** only shipping configurations compile and no fault instrumentation enters exported binaries

#### Scenario: Explicit Local Extended Validation
- **WHEN** an operator requests the local extended suite
- **THEN** all-features, no-default-features and declared WebAssembly compatibility proof remain available alongside preserved instrumented stories
- **AND** ordinary canonical-capable cases remain in provider CI

#### Scenario: Compile Release Compatibility Proofs
- **WHEN** an operator explicitly requests local extended validation with the release profile
- **THEN** compatibility selectors remain available with the release-tests compiler profile
- **AND** release provider pipelines do not invoke that extended matrix implicitly
