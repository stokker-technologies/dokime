# Data Root Identity Delta

## MODIFIED Requirements

### Requirement: Storage Format Refusal

The product SHALL refuse unreadable product-owned persistent state and incompatible
database layouts with backup and explicit fresh-root/reset guidance, preserving
existing data. Plugin evidence schema changes and obsolete source-digest markers
SHALL NOT invalidate a readable root. No automatic deletion, reset or migration
SHALL occur; development schema/evidence baselines SHALL remain v1.
This reconciles the older format-marker requirement with the main
`data-shape-identity` requirements and the landed runtime.

#### Scenario: Matching Root

- **WHEN** a backend opens a root with readable product-owned state and a compatible database layout
- **THEN** it starts normally without replacing stored evidence

#### Scenario: New Root

- **WHEN** a backend opens an empty root
- **THEN** it establishes its product-owned layout at baseline v1 before admitting evidence

#### Scenario: Different Format

- **WHEN** a backend cannot decode a root's product-owned state or validate its database layout
- **THEN** startup refuses with backup/fresh-root or explicit-reset guidance and preserves existing data

#### Scenario: Populated Root Without a Record

- **WHEN** a populated root has no obsolete format-marker record
- **THEN** marker absence alone does not refuse a readable product root; an incompatible foreign layout is refused and preserved

#### Scenario: Readable Root with Obsolete Marker

- **WHEN** a readable root contains an obsolete evidence-format/source-digest marker
- **THEN** it opens and the marker and historical evidence remain unchanged

#### Scenario: Unreadable Product State or Foreign Root

- **WHEN** startup encounters unreadable product-owned state or an incompatible populated root
- **THEN** it refuses with backup and fresh-root/explicit-reset guidance without deleting or replacing existing data
- **AND** the operator explicitly selects new storage or performs a separately authorized reset

### Requirement: Derived Format Identity

Describable stream/record shape identities SHALL derive from their schema and be
compared only for equality. Product storage compatibility SHALL depend on the
product's ability to decode its persistent state and validate its database layout,
not a whole-build source/codec digest or plugin-defined shape changes.

#### Scenario: Stored Layout Changes

- **WHEN** a stored product type or database layout changes
- **THEN** the product checks whether existing persistent state remains readable and refuses incompatible state without resetting it
- **AND** a plugin shape change receives a derived equality identity while retained evidence keeps its original definition

#### Scenario: Unrelated Change

- **WHEN** a build changes no persistent product encoding or layout
- **THEN** readable roots continue to open without a whole-build digest gate

#### Scenario: Codec Source Trivia

- **WHEN** only codec comments or whitespace change
- **THEN** readable roots continue to open and their evidence is preserved

#### Scenario: Codec Source Changes

- **WHEN** codec source changes while encoded product state remains decodable
- **THEN** readable roots continue to open; source changes alone do not demand replacement storage
- **AND** actual unreadable product state is refused without automatic migration or deletion

#### Scenario: Plugin Schema or Unrelated Source Changes

- **WHEN** plugin schemas or unrelated source change while product-owned state remains readable
- **THEN** the root continues to open and saved evidence retains its captured definitions
- **AND** no authored evidence version or build digest is used to demand a reset
