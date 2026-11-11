## MODIFIED Requirements

### Requirement: Actionable Startup Failure

Plugin entry failures SHALL retain structured root causes and consequences without Rust
Debug formatting. Managed failures SHALL name the installed bundle; remote unavailability
SHALL name the URL. Settings and secret failures SHALL name their remediation without
startup settings injection, generation comparisons or Reload admission.

#### Scenario: Missing Secret

- **WHEN** a plugin operation requests an unavailable secret
- **THEN** the failure names its identifier and offers Secret Storage remediation

#### Scenario: Dependent Work

- **WHEN** a call needs a provider that is unavailable
- **THEN** its failure names that provider and leaves unrelated entries available

#### Scenario: Independent Failures

- **WHEN** several entries have independent connection or startup failures
- **THEN** each entry retains its root cause and expandable structured details

#### Scenario: Missing Resources Alone

- **WHEN** an operation fails solely because a required resource is missing
- **THEN** the missing resource remains the visible root cause
