# CI Migration Specification
## Purpose
Preserve Dokime's native validation, artifact ownership and protected delivery using editable Hub CI baselines. Dokime owns its CI workflows, actions and tooling.

## Requirements
### Requirement: Use the Ordinary CI Job Budget
Ordinary CI jobs SHALL inherit the canonical 120-minute default without unnecessary per-job overrides. Exceptions SHALL have a concrete hang/deadlock justification and thirty-minute increments. Individual test deadlines and required checks SHALL remain unchanged.

#### Scenario: Render Required Jobs
- WHEN the candidate regenerates its native CI graph
- THEN ordinary jobs use the canonical 120-minute budget
- AND test failures remain blocking within their existing story deadlines

### Requirement: Use GitHub-Hosted Runners
Every GitHub Actions job SHALL use the matching GitHub-hosted platform default, including crate compilation, coverage, benchmark and release jobs. Root and generated starter configurations SHALL NOT select local runners. Required native platforms, declared bootstrap, job budgets, individual test deadlines, Release Rehearsal and release preparation profiles and behavior SHALL remain unchanged.

#### Scenario: Render Producer and Consumer Routing
- **WHEN** native CI renders the actual task graph
- **THEN** all producer, consumer, coverage, benchmark and release jobs use matching GitHub-hosted platforms
- **AND** no job or legacy repository routing variable redirects execution to local runners
