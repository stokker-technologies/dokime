# Report Selection Sharing

## ADDED Requirements

### Requirement: One Captured Tree Per Equivalent Direct Selection

Equivalent direct report plot selections SHALL share one immutable captured value tree. Each plot SHALL retain unique metadata ownership and serialize the shared tree by reference without changing the report JSON.

#### Scenario: Multiple Cards Request The Same Selection

- **WHEN** eight cards request an equivalent direct telemetry selection
- **THEN** their captured values retain the same tree owner
- **AND** each result can transfer its metadata independently
- **AND** all original points remain available for zoom, even when the display point budget is smaller

#### Scenario: Shared Values Are Serialized And Read Back

- **WHEN** a shared captured value contains signed and unsigned integer boundaries
- **THEN** serialization matches the corresponding owned value exactly
- **AND** reading that JSON preserves numeric equality
