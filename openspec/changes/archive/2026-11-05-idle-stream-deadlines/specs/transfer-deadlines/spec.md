## ADDED Requirements

### Requirement: Transfers Use Operation Silence Budgets
Uploads and downloads SHALL use their operation timeout as maximum silence, preserving size limits.

#### Scenario: Artifact Upload Progress
- **WHEN** an artifact upload moves frames within timeout for longer than the total timeout
- **THEN** it succeeds

#### Scenario: Plugin Install Progress
- **WHEN** a plugin install moves frames within timeout for longer than the total timeout
- **THEN** it succeeds

#### Scenario: Transfer Stalls
- **WHEN** an upload or final response wait is silent beyond policy timeout
- **THEN** it fails with DeadlineExceeded

#### Scenario: Download Policy
- **WHEN** a client generic request timeout differs from download operation policy
- **THEN** frame silence uses the operation policy
