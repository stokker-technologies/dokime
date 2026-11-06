## MODIFIED Requirements

### Requirement: Host Native Probe Settings

Host native probe tunables SHALL be declared through plugin settings in `settings.rs`, with validated bounds and defaults preserving current behavior. Settings SHALL resolve at plugin startup and apply on explicit Reload or backend startup. Each observer SHALL resolve its collection interval from the prepared settings snapshot on activation, including activation after Reload.

#### Scenario: Configured Bluetooth Cache Refresh

- **WHEN** a configured deep-refresh interval has not elapsed
- **THEN** the collector reuses its cached snapshot without probing again

#### Scenario: Refresh Becomes Due

- **WHEN** the configured deep-refresh interval elapses
- **THEN** the collector probes again and updates its cache

#### Scenario: Invalid Native Probe Setting

- **WHEN** a saved timing is zero or exceeds its declared bound
- **THEN** settings resolution refuses it before collector startup
