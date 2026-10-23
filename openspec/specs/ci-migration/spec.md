# CI Migration Specification

## Purpose
Preserve Dokime's native validation, artifact ownership and protected delivery while adopting the shared Hub CI contract.

## Agreed Migration Scope

The user approved fixing migration-related convention findings, required canonical task/pin configuration and Hub false positives while deferring unrelated pre-existing cleanup. The broad convention audit remains failing; this migration does not claim otherwise. Deferred categories are skeleton/header and attributed-member spacing, test-domain mirroring and placement, existing identifier naming, Error module placement and test environment access, Rustdoc examples, dependency grouping outside the new executable crates, and unchanged Apache license text. Representative unchanged sources include cargo-dokime/src/cli.rs, runtime test service state modules and LICENSE-APACHE. Required CI, packaging and publication dry-run failures remain blocking.

Timed benchmarks are optional and explicitly deferred from migration and release gates. The benchmark producer passed; unit measurement was interrupted under native CI contention, and integration/system measurements were not started. Partial output is excluded from accepted baselines; no completed timing or regression result is claimed. Future measurements require a quiet window and all declared layers. The producer's unused implicit library harness is a deferred optimization opportunity; preserve the current producer and five explicit benchmark executables.

## Requirements

### Requirement: Adopt The Agreed Hub Contract
The product SHALL migrate its authored tasks and configuration and canonically generated tooling/workflows to the locked Hub CI contracts while preserving required native platform coverage, artifact dependencies and release behavior.

#### Scenario: Candidate Delivery
- **WHEN** complete source passes applicable local proof
- **THEN** publish the candidate PR to dev, monitor and repair required provider checks until green, merge with protections enforced and verify the merged revision.

### Requirement: Honest Baseline And Repairs
The initial full baseline SHALL run before implementation as explicitly requested; every observed existing failure remains actionable and cannot waive final gates. Only targeted proof runs during migration until source closes. Optional nightly workloads are excluded unless explicitly requested.

#### Scenario: Baseline Failure
- **WHEN** an initial lane fails or a prerequisite prevents it running
- **THEN** record the actual result, repair the cause within the authorized migration and prove required behavior before delivery.

### Requirement: Separate Executable Publication

The dokime and dokime-server libraries SHALL retain their published names and APIs. Their executable targets SHALL belong to non-publishable CLI crates while retaining existing executable names and behavior.

#### Scenario: Library And Binary Delivery

- WHEN the candidate builds, tests and packages Dokime
- THEN library consumers resolve the existing library packages and executable process stories resolve the new binary owners
- AND registry publication excludes the CLI packages while release assets retain dokime and dokime-server executables

### Requirement: Publish The Distinct Python Client
The Python distribution SHALL be dokime-client and expose only the dokime_client import package. Installation documentation SHALL use uv. Rust crate, protocol, product and executable names SHALL remain unchanged.

#### Scenario: Install And Import The Client
- WHEN a user installs the built dokime-client wheel through uv
- THEN dokime_client and its generated wire modules import with the existing client behavior
- AND the wheel metadata names dokime-client and contains no old dokime import package

#### Scenario: Reproduce And Publish The Client
- WHEN the producer regenerates bindings, builds distributions and packages the candidate
- THEN the renamed uv workspace and artifact consumers use the same distribution/import contract
- AND the existing PyPI publisher expects stokker-technologies/dokime, managed-release.yml and the release environment

### Requirement: Use The Ordinary CI Job Budget
Ordinary CI jobs SHALL inherit the canonical 120-minute default without unnecessary per-job overrides. Exceptions SHALL have a concrete hang/deadlock justification and thirty-minute increments. Individual test deadlines and required checks SHALL remain unchanged.

#### Scenario: Render Required Jobs
- WHEN the candidate regenerates its native CI graph
- THEN ordinary jobs use the canonical 120-minute budget
- AND test failures remain blocking within their existing story deadlines

### Requirement: Reserve Local Runners For Crate Compilation
Jobs that compile crates, including nested scaffold builds, instrumented coverage and Cargo publication verification, SHALL use local native runners. Jobs that consume existing artifacts without crate compilation SHALL use matching hosted runners. Release Rehearsal and release preparation SHALL retain their existing release profile and behavior.

#### Scenario: Render Producer And Consumer Routing
- WHEN native CI renders the actual task graph
- THEN Rust Check, Product, Tests and Bench producers, Rust acceptance with generated-project builds, Rust coverage and Release Dry, Rehearsal and Tag use local runners
- AND artifact-only tests, benchmark execution, packaging and other noncompiling jobs use matching hosted platforms

### Requirement: Validate In The Assigned Worktree
All local implementation and validation SHALL use the assigned worktree. Local proof SHALL NOT create an isolated checkout, exported source tree or provider simulator. Before the next candidate push, the full applicable local graph SHALL pass on available native platforms after the combined source closes. Actual native CI runners SHALL supply distinct-path archive proof and required proof for unavailable local platforms.

#### Scenario: Deliver An Archive Portability Repair
- WHEN the candidate completes its source and applicable local validation in the assigned worktree
- THEN the next PR update starts required native runner checks with archive workspace remapping
- AND integration remains blocked until distinct-path consumers and every required native platform pass
