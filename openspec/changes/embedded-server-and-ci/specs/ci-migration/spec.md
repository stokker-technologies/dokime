# CI Migration

## MODIFIED Requirements

### Requirement: Separate Executable Publication
The dokime and dokime-server libraries SHALL retain their published identities. The non-publishable dokime-cli crate SHALL own the dokime executable, including dokime server. The non-publishable GUI SHALL embed the shared server library. No standalone dokime-server executable or compatibility wrapper SHALL ship.

#### Scenario: Library and Binary Delivery
- **WHEN** the candidate builds, tests and packages Dokime
- **THEN** library consumers resolve the existing libraries and headless process stories invoke dokime server
- **AND** registry publication excludes CLI and GUI crates while release assets contain the remaining declared executable owners

### Requirement: Validate in the Assigned Worktree
Local implementation SHALL use the assigned worktree and focused project tasks. Hosted CI and Nightly SHALL provide final proof on every required native platform. Windows proof SHALL NOT substitute for Linux proof. Authorized dev integration SHALL remain ungated.

#### Scenario: Deliver an Archive Portability Repair
- **WHEN** a source-complete candidate carries an archive portability repair
- **THEN** hosted native checks exercise transported consumers on both required platforms
- **AND** final proof remains open for any missing or failing platform result

#### Scenario: Native Proof Is Missing
- **WHEN** one required platform has no result
- **THEN** that proof remains open and no cross-platform pass is claimed

### Requirement: Share Native Compilation Ownership
Product and Tests SHALL compile separately per native platform/profile using Cargo defaults and explicit artifact namespaces. Clippy SHALL run in lint tasks. Bench SHALL use the default bench profile. Tests and packages SHALL consume existing declared exports. Project compiler profile overrides SHALL be absent; shipping behavior and coverage SHALL remain intact.

#### Scenario: Build a Native Candidate
- **WHEN** native CI builds the candidate
- **THEN** Product and canonical Tests producers run independently, and consumers use explicit artifact inputs
- **AND** compatible compiler outputs may be reused without promising one compilation for distinct targets
- **AND** warm-cache whole pipelines remain below 45 minutes, targeting 30, and Tests Build is no longer than Product Build; breaches receive phase-level investigation without silent coverage reduction

#### Scenario: Build Shipping Artifacts Without Tests
- **WHEN** an ordinary Product or installer build runs
- **THEN** only shipping configurations compile and no fault instrumentation enters exported binaries

#### Scenario: Explicit Local Extended Validation
- **WHEN** an operator explicitly requests local extended validation
- **THEN** the declared WebAssembly proof and instrumented stories remain available; removed all-features and no-default-features matrices are not restored
- **AND** ordinary canonical-capable cases remain in provider CI

#### Scenario: Compile Release Compatibility Proofs
- **WHEN** an operator explicitly requests local extended validation with the release graph profile
- **THEN** compatibility selectors use Cargo's default unoptimized test profile while retaining release artifact paths
- **AND** release provider pipelines do not invoke that extended matrix implicitly
