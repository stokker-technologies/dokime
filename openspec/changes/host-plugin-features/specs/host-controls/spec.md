# Host Controls

## ADDED Requirements

### Requirement: Invertible Radio And Adapter Controls

`host.network.radio.set` and `host.network.adapter.set` SHALL be invertible transactions whose inverse restores the prior state; every change made during an execution SHALL be recorded with timestamps and inverted at run end, and pending inverses SHALL be applied first when the backend starts after a crash.

#### Scenario: Blackout Window

- **WHEN** a step switches the Wi-Fi radio off and the execution ends by completion, cancellation or failure
- **THEN** the radio is restored and the report shows the off and restore timestamps

#### Scenario: Crash Recovery

- **WHEN** the backend restarts after a crash with a recorded, uninverted change
- **THEN** the inverse is applied before anything else starts

#### Scenario: Adapter Without Administrator

- **WHEN** `host.network.adapter.set` runs without administrator rights on Windows
- **THEN** it refuses with a "requires administrator" error and no elevation prompt appears

#### Scenario: Active Connection Guard

- **WHEN** a step would switch off the radio or adapter carrying the active remote client or backend connection without explicitly allowing it
- **THEN** the change is refused

### Requirement: Power Plan And CPU Capacity

The host plugin SHALL read the active power plan, set it as an invertible transaction, and report CPU load as an absolute GFLOP/s measure derived from a documented calibration.

#### Scenario: Power Plan Restored

- **WHEN** a step sets a power plan and the execution ends
- **THEN** the previous plan is active again

