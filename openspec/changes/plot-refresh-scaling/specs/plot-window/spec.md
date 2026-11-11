## MODIFIED Requirements

### Requirement: View Refresh Ownership

Live viewers SHALL retain Refresh left of the shared window slider, progress, disabled state until batch completion, Updated hh:mm:ss on success and existing failure notifications. Entry or Refresh SHALL authorize one batch for all plots, scheduled visible-first with bounded concurrency. All SHALL use one saved cut per batch, allow deferred reads at that cut and never start a fresh batch from visibility, zoom or evidence changes.

#### Scenario: Manual All

- **WHEN** All is entered and telemetry, visibility, zoom or document updates subsequently arrive
- **THEN** only the entry batch loads, deferred cards read its original saved cut, no live watch starts and only Refresh or another view entry authorizes a new batch

#### Scenario: Timed Refresh

- **WHEN** Refresh is pressed in a timed view with off-screen plots
- **THEN** one fresh batch covers every plot with visible plots served first, off-screen one-shot work stays bounded and only on-screen plots retain automatic live interests

#### Scenario: Refresh Completion

- **WHEN** an authorized batch is pending, succeeds or fails
- **THEN** pending progress disables Refresh, completion advances the successful update time and failure permits retry through existing notifications without advancing that time
