## ADDED Requirements

### Requirement: Derived Data Shape Identities
Data shape identities SHALL be derived with Harmos and SHALL be compared only for equality. API, release and algorithm behavior versions SHALL remain unchanged.

#### Scenario: Shape Changes
- **WHEN** a stream row, record or snapshot schema changes
- **THEN** its identity derives from that shape without an authored version

### Requirement: Root Compatibility Belongs to the Product
Only an undecodable Dokime-owned persistent type SHALL refuse opening a storage root, naming the unreadable state and advising a fresh root. Plugin-defined data SHALL retain stored definitions and SHALL NOT invalidate the root. No data SHALL be reset or migrated.

#### Scenario: Plugin Evolution
- **WHEN** an execution publishes evidence and seals, and the plugin stream and record shapes change before reopening the same root
- **THEN** the root opens and the old execution's report builds from captured definitions

#### Scenario: Built-In Incompatibility
- **WHEN** a built-in persistent journal record cannot be decoded
- **THEN** startup refuses with the record name and fresh-root advice, preserving stored data

#### Scenario: Unchanged Restart
- **WHEN** an execution is published, sealed and reported and the root reopens with unchanged shapes
- **THEN** stored execution evidence remains readable
