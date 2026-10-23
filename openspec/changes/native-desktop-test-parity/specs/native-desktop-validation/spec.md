## ADDED Requirements

### Requirement: Native Desktop Scenario Parity

Ordinary desktop system validation SHALL execute the same scenarios on Windows and Linux, including local Windows validation before a PR update. Platform-specific driver setup SHALL NOT omit ordinary scenarios.

#### Scenario: Local Windows Validation

- **WHEN** the developer runs the applicable local validation on Windows
- **THEN** the native desktop UI journeys execute rather than reporting a platform skip

#### Scenario: Both CI Platforms

- **WHEN** required desktop system jobs run on declared Windows and Linux runners
- **THEN** each platform executes the shared ordinary journey inventory using its native driver

### Requirement: Deterministic Windows Driver Bootstrap

Windows desktop bootstrap SHALL obtain a driver matching the installed WebView2 runtime and verify its version before executing UI tests.

#### Scenario: Missing Driver

- **WHEN** the runner has WebView2 but lacks its matching driver
- **THEN** shared bootstrap provisions and verifies that driver without relying on a manually configured local runner

### Requirement: Stopping Provider Groups Settle

Provider completion SHALL NOT resubmit a pending group while the request is stopped or the runtime is shutting down. Valid non-stopping replay SHALL remain idempotent.

#### Scenario: Cancellation With Pending Work

- **WHEN** a cancelled provider invocation leaves pending work and additional groups
- **THEN** completion settles the request without repeatedly submitting the same cancelled group

#### Scenario: Replay During Shutdown

- **WHEN** an advance transaction is checked with pending work during shutdown or a stop request
- **THEN** the transaction refuses advancement

### Requirement: Independent Starter Tool Resolution

Generated starter validation SHALL resolve its declared tools without machine or system mise configuration, while preserving installed data/cache homes and the normal temporary location.

#### Scenario: Hostile Machine Tool Declarations

- **WHEN** global and system configuration declare unrelated private tools
- **THEN** an offline starter resolution uses only the starter's declared tools
- **AND** no private tool lookup or installation is required
