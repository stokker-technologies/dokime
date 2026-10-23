## REMOVED Requirements

### Requirement: Reserve Local Runners For Crate Compilation
**Reason**: The user replaced local runner routing with GitHub-hosted execution for every job.
**Migration**: Remove local job runner overrides and legacy routing variables, then regenerate the root and starter workflows with hosted defaults.

## ADDED Requirements

### Requirement: Use GitHub-Hosted Runners
Every GitHub Actions job SHALL use the matching GitHub-hosted platform default, including crate compilation, coverage, benchmark and release jobs. Root and generated starter configurations SHALL NOT select local runners. Required native platforms, declared bootstrap, job budgets, individual test deadlines, Release Rehearsal and release preparation profiles and behavior SHALL remain unchanged.

#### Scenario: Render Producer And Consumer Routing
- **WHEN** native CI renders the actual task graph
- **THEN** all producer, consumer, coverage, benchmark and release jobs use matching GitHub-hosted platforms
- **AND** no job or legacy repository routing variable redirects execution to local runners
