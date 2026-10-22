# CI Migration

## ADDED Requirements

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
