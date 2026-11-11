## MODIFIED Requirements

### Requirement: Incremental Live Updates

A plot watch SHALL open with a snapshot and SHALL NOT delay drawing for exact statistics. Refresh policy SHALL follow per-kind desktop benchmark evidence: cheap bounded kinds default to invalidation/refetch of the visible window; time-series bucket deltas remain only where measured bandwidth justifies them. Both paths SHALL preserve the captured cut and late-sample correctness.

#### Scenario: Bounded Replacement Kind

- **WHEN** relevant samples change a kind assigned invalidation/refetch
- **THEN** the visible plot requests a fresh bounded result without pushed bucket state

#### Scenario: Late Sample on a Delta Path

- **WHEN** a late sample changes an already delivered time-series bucket on a measured delta path
- **THEN** the watch replaces that bucket with the correct result at the new cut

#### Scenario: Reconnect

- **WHEN** a client reconnects
- **THEN** it receives a fresh snapshot and continues using its kind's measured policy
