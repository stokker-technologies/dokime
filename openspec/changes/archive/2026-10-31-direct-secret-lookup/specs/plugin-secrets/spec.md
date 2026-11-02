# Plugin Secrets

## ADDED Requirements

### Requirement: Resolve Declared Store Identifiers

The runtime SHALL resolve each secret declaration's identifier directly in shared
Secret Storage before plugin start, without bindings or environment sources.

#### Scenario: Required Identifier Exists
- **WHEN** the declared required identifier exists
- **THEN** initialization receives its protected value under that same identifier

#### Scenario: Required Identifier Is Missing
- **WHEN** a required identifier is absent
- **THEN** startup fails before launch with a typed error naming the identifier and instructing the user to add it and restart

#### Scenario: Optional Identifier Is Missing
- **WHEN** an optional declared identifier is absent
- **THEN** initialization receives absence

#### Scenario: Retry Reads New Secrets
- **WHEN** a missing required secret is added and the plugin is retried or Dokime restarted
- **THEN** initialization reads the store again and succeeds

### Requirement: Show Declarations Without Bindings

The GUI SHALL show declared identifiers and availability read-only, with an Add In
Secret Storage action opening Add Secret prefilled with the declaration identifier.

#### Scenario: Missing Declaration Is Added
- **WHEN** the operator selects Add In Secret Storage for a missing identifier
- **THEN** Add Secret opens with that identifier prefilled and no per-plugin binding is saved

#### Scenario: Catalog References Include Failed Plugins
- **WHEN** an installed plugin declares a stored identifier even if startup failed
- **THEN** Secret Storage lists that plugin in References

### Requirement: Discard Saved References

Loading settings SHALL discard removed secret references without error, retaining
ordinary settings and protected store values without migration or version bumps.

#### Scenario: Old References Are Present
- **WHEN** saved settings contain old stored-secret or environment reference objects
- **THEN** those references are ignored and startup uses declaration identifiers
