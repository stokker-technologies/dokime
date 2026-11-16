## Purpose

Provide independent protocol consumers with one complete release contract source archive that needs no separate library proto download.

## ADDED Requirements

### Requirement: One Self-Contained Archive Per Release

Every Dokime release SHALL ship one contracts archive containing Dokime protos, the pinned Harmos carrier protos, all recursive imports including `google/*`, applicable licenses and protoc/Buf instructions and configuration. No separate Harmos download or undocumented include tree SHALL be required.

#### Scenario: Independent Protocol Consumer

- **WHEN** a consumer unpacks the archive and follows its instructions with the documented compiler installed
- **THEN** protoc and Buf resolve all contract imports from the archive's sources without fetching another proto tree

### Requirement: Packaging Verifies Import Closure

Packaging SHALL verify every shipped proto import recursively against archive members and reject missing or conflicting definitions. The archive and embedded export SHALL use the same contract inventory and upstream revision.

#### Scenario: Missing Google Import

- **WHEN** packaging omits a Google proto imported by a shipped contract
- **THEN** packaging fails naming the unresolved import and the action to include its source and license

#### Scenario: Successful Release Archive

- **WHEN** a release archive is packaged
- **THEN** closure verification and the supplied compiler instructions succeed using its own proto sources
