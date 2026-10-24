# CI Migration

## MODIFIED Requirements

### Requirement: Separate Executable Publication
The dokime and dokime-server libraries SHALL retain their published identities. The non-publishable dokime-cli crate SHALL own the dokime executable, including dokime server. The non-publishable GUI SHALL embed the shared server library. No standalone dokime-server executable or compatibility wrapper SHALL ship.

#### Scenario: Library And Binary Delivery
- **WHEN** the candidate builds, tests and packages Dokime
- **THEN** library consumers resolve the existing libraries and headless process stories invoke dokime server
- **AND** registry publication excludes CLI and GUI crates while release assets contain the remaining declared executable owners

### Requirement: Validate In The Assigned Worktree
Local implementation SHALL use the assigned worktree without a separate source checkout. After combined source closure, the canonical verified ci:simulate gate SHALL execute the rendered job graph and artifact dependencies across every declared required platform. A platform filter SHALL represent partial proof. Native provider checks SHALL remain required before protected integration.

#### Scenario: Deliver An Archive Portability Repair
- **WHEN** the candidate completes combined source and the full local simulator gate
- **THEN** the next PR update starts required hosted native checks with archive workspace remapping
- **AND** integration remains blocked until transported consumers and every required native platform pass

### Requirement: Share Native Compilation Ownership
Product and Tests SHALL compile independently in parallel per platform/profile, under repository-owned canonical configurations and matching Clippy. Product SHALL use default Cargo dev or release; all Tests, matching Clippy and local extended configurations SHALL use default unoptimized test. Project compiler profile overrides SHALL be absent. Bench SHALL use default bench. The graph artifact namespace SHALL remain explicit and independent of the Cargo test output directory. Shipping behavior and canonical coverage SHALL remain intact. Check SHALL not compile; Test and Package SHALL consume existing exports. Noncanonical compatibility and fault injection SHALL remain explicit local-only extended validation.

#### Scenario: Build A Native Candidate
- **WHEN** native CI builds the candidate
- **THEN** Product and canonical Tests producers run independently in parallel, and consumers use explicit artifact inputs
- **AND** compatible compiler outputs may be reused without promising one compilation for distinct targets
- **AND** warm-cache whole pipelines remain below 45 minutes, targeting 30, and Tests Build is no longer than Product Build; breaches receive phase-level investigation without silent coverage reduction

#### Scenario: Build Shipping Artifacts Without Tests
- **WHEN** an ordinary Product or installer build runs
- **THEN** only shipping configurations compile and no fault instrumentation enters exported binaries

#### Scenario: Explicit Local Extended Validation
- **WHEN** an operator explicitly requests local extended validation
- **THEN** all-features, no-default-features and declared WebAssembly compatibility proof remain available alongside preserved instrumented stories
- **AND** ordinary canonical-capable cases remain in provider CI

#### Scenario: Compile Release Compatibility Proofs
- **WHEN** an operator explicitly requests local extended validation with the release graph profile
- **THEN** compatibility selectors use Cargo's default unoptimized test profile while retaining release artifact paths
- **AND** release provider pipelines do not invoke that extended matrix implicitly
