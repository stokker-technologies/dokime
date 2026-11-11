## ADDED Requirements

### Requirement: Saved-Evidence Card Freshness

Live document cards SHALL consume the shared scoped change source and refresh affected visible bindings when durable telemetry or record frontiers advance, without requiring runtime state changes. Accepted-live progress SHALL NOT masquerade as saved evidence. Structure SHALL remain separate; completed card content SHALL remain during reads. Frozen reports SHALL never subscribe or advance their sealed cut.

#### Scenario: Samples Without State Changes

- **WHEN** telemetry is saved for a visible summary card while execution state remains unchanged
- **THEN** that card refreshes at the new durable cut and unrelated cards and document structure are not requeried

#### Scenario: Accepted but Unsaved Samples

- **WHEN** live plot progress advances before the matching evidence becomes durable
- **THEN** saved-evidence cards wait for durable progress while timed plots may display the accepted tail

#### Scenario: Recover a Missed Save

- **WHEN** a card resumes after missing a saved-progress notification
- **THEN** frontier comparison discovers the change and refreshes affected live bindings while frozen report content stays fixed
