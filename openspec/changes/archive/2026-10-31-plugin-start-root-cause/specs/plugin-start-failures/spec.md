## ADDED Requirements

### Requirement: Actionable Startup Failure

The plugin startup failure SHALL show root causes using Display or structured fields, retain all diagnostics, summarize dependent unavailable work and offer the existing settings/retry flow.

#### Scenario: Missing Secret
- **WHEN** plugin startup is refused because a secret identifier is unavailable
- **THEN** the failure carries its structured identity and offers Set Secret opening Secret Storage Add Secret with the identifier prefilled, followed by the existing restart retry

#### Scenario: Dependent Work
- **WHEN** a startup/prepare pass reports missing resources alongside a construction failure
- **THEN** root prose excludes the consequences and expandable details preserve each consequence

#### Scenario: Independent Failures
- **WHEN** startup has independent root failures
- **THEN** each root remains visible without Rust Debug formatting

#### Scenario: Missing Resources Alone
- **WHEN** a pass contains missing resources without a construction failure
- **THEN** those failures remain visible roots
