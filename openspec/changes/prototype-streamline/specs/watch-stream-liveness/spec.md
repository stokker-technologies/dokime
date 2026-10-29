# Watch Stream Liveness

## ADDED Requirements

### Requirement: Heartbeats Survive Backpressure

Quiet plot and log watches SHALL retain their configured heartbeat deadline while an offered frame waits for acknowledgement.

#### Scenario: The Consumer Requests Its Next Frame After A Quiet Interval

- **WHEN** the previous frame has been offered and the configured heartbeat interval elapses before its send completes
- **THEN** the next read receives a keepalive without starting another whole heartbeat interval
- **AND** both watches continue delivering actual updates after the quiet interval
