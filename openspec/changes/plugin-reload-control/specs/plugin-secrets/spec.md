## MODIFIED Requirements

### Requirement: Resolve Declared Store Identifiers

The runtime SHALL resolve each secret declaration's identifier directly in shared
Secret Storage before plugin start, without bindings or environment sources.

#### Scenario: Required Identifier Exists
- **WHEN** the declared required identifier exists
- **THEN** initialization receives its protected value under that same identifier

#### Scenario: Required Identifier Is Missing
- **WHEN** a required identifier is absent
- **THEN** startup fails before launch with a typed error naming the identifier and instructing the user to add it and reload the plugin

#### Scenario: Optional Identifier Is Missing
- **WHEN** an optional declared identifier is absent
- **THEN** initialization receives absence

#### Scenario: Retry Reads New Secrets
- **WHEN** a missing required secret is added and the plugin is reloaded or Dokime restarted
- **THEN** initialization reads the store again and succeeds
