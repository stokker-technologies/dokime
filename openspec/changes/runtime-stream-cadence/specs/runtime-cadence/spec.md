## ADDED Requirements

### Requirement: Host Cadence Uses Plugin Settings

Every host collection interval SHALL have one bounded declaration in settings.rs.
The runtime SHALL use the settings resolved at plugin startup.

#### Scenario: Saved Cadence Is Applied

- **WHEN** a plugin starts with a saved nondefault collection interval
- **THEN** its observer samples using that interval until restart

#### Scenario: Invalid Cadence Is Refused

- **WHEN** a saved collection interval is outside declared bounds
- **THEN** settings resolution refuses startup

### Requirement: Static Contracts Do Not Own Cadence

SDK observer inventories SHALL retain only static identity, outputs and activation.
All SDK consumers SHALL use the runtime cadence API without a compatibility shim.

#### Scenario: Runtime Resource Supplies Cadence

- **WHEN** the producing plugin activates an observer
- **THEN** the runtime uses its required positive declared interval unless an authored
  method overrides it from execution resources
- **AND** schema and replay window contracts retain their static definitions
