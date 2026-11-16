## Purpose

Keep Dokime plugin selection, installation, admission and durable administration in the product while reusing Harmos activation machinery.

## ADDED Requirements

### Requirement: Product Owns Installed Plugins

Dokime SHALL own plugin installation, registry, bundle format/build/verification, manifests, artifact digests, distribution, persisted settings and plugin management operations and their SDK projections. Its registry SHALL explicitly select and load installed plugins at startup through the Harmos typed facade, without a `PluginBackend` trait.

#### Scenario: Restart With Installed Plugins

- **WHEN** Dokime restarts with installed plugins and saved settings
- **THEN** its registry resolves their placements/settings and calls Harmos load for each selected entry; Harmos performs only runtime activation

### Requirement: Product Kind Admits Its Contract

Dokime SHALL register its plugin kind with product admission and settings-resolution callbacks. After generic validation it SHALL validate product declaration sections and resolve effective settings before Harmos initializes. A refusal SHALL publish no activation and SHALL propagate the load's cancellation and deadline policy.

#### Scenario: Invalid Suite Declaration

- **WHEN** carrier descriptors are valid but a suite references an invalid product declaration
- **THEN** Dokime admission refuses before initialize, and Harmos settles the candidate's owned resources

### Requirement: Durable Administration Preserves Atomicity

Product management SHALL preserve conditional revisions, atomic publication, durable cleanup/recovery ownership and replayable receipts across registry, installation and settings changes. A rejected candidate SHALL NOT replace the admitted entry, and replaying an accepted mutation SHALL return its original receipt.

#### Scenario: Replacement Candidate Fails

- **WHEN** a replacement fails verification or admission before publication
- **THEN** the prior installed entry and settings remain authoritative and the failed candidate's resources are settled

#### Scenario: Accepted Settings Mutation Is Replayed

- **WHEN** the same mutation identity and normalized settings input are submitted again
- **THEN** Dokime returns the original receipt and revision without applying another change

### Requirement: Clean Storage Break Preserves Existing Roots

Dokime SHALL keep development storage at baseline v1 and refuse incompatible roots with an action to select a fresh root. It SHALL NOT migrate, reset or delete existing roots as part of this cutover.

#### Scenario: Earlier Registry Shape

- **WHEN** startup encounters an incompatible saved plugin registry or settings shape
- **THEN** startup refuses with the fresh-root action and leaves existing bytes intact
