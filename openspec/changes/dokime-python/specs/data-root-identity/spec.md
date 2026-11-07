## ADDED Requirements

### Requirement: Storage Format Refusal
Every backend SHALL refuse to open a data root written by a different storage format,
without changing its contents.

#### Scenario: Matching Root
- **WHEN** a backend opens a root recorded with its own format
- **THEN** it starts normally

#### Scenario: New Root
- **WHEN** a backend opens an empty root
- **THEN** it records its format and writer build before writing evidence

#### Scenario: Different Format
- **WHEN** a backend opens a root recorded with another format
- **THEN** startup fails with `StorageFormatMismatch`, naming the root and both builds, and
  no file changes

#### Scenario: Populated Root Without a Record
- **WHEN** a backend opens a root with data but no format record
- **THEN** startup fails the same way

### Requirement: Derived Format Identity
The format identity SHALL be derived from the declared stored schemas at build time.

#### Scenario: Stored Layout Changes
- **WHEN** a stored type's layout changes
- **THEN** the identity changes with no manual step

#### Scenario: Unrelated Change
- **WHEN** a build changes no stored layout
- **THEN** the identity is unchanged and its roots keep opening
