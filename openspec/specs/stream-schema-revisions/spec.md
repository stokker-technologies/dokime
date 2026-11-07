# stream-schema-revisions Specification

## Purpose
TBD - created by archiving change stream-schema-revisions. Update Purpose after archive.

## Requirements

### Requirement: Automatic Immutable Row Revisions
Dokime SHALL derive row revisions from schemas, retain each definition and decode saved rows through their own revisions. Authors SHALL NOT bump versions or restart the backend for a changed row.

#### Scenario: Reload Within One Cycle
- **WHEN** executions capture different row definitions under one stream id with live reload between them in one cycle
- **THEN** both execution reports and their cycle report build with the correct values
- **AND** current catalogs and plots resolve the activated stream by id

#### Scenario: Restart on One Root
- **WHEN** two executions capture different row definitions with a backend restart between them on one storage root
- **THEN** both execution reports build with their correct values
- **AND** recovery seals the idle pre-restart cycle
- **AND** a new cycle after restart supports mixed revisions through live reload, with correct execution and cycle report values

### Requirement: Cross-Revision Columns
Queries SHALL select streams by id and column, omit absent columns, and isolate incompatible column failures to affected report bindings with diagnostics naming stream and column. Conflicting definitions under one stored key SHALL be refused with fresh-root guidance without deleting data.

#### Scenario: Missing and Incompatible Columns
- **WHEN** a cycle report spans revisions with absent or incompatibly typed columns
- **THEN** readable values render and incompatible bindings state the affected stream and column

#### Scenario: Ambiguous Evidence
- **WHEN** two different stored definitions share one key
- **THEN** reading refuses to choose one and explains that a fresh storage root is required

### Requirement: Terminal Finalization Presentation
Finalization failure SHALL persist and settle through the same terminal state path as success. GUI errors SHALL name a useful action and SHALL NOT offer futile retry. Steps SHALL follow the frozen plan order.

#### Scenario: Failed Report Survives Reload
- **WHEN** background finalization fails and the execution is read again after reload
- **THEN** Control and sidebar show a terminal outcome with a useful action and no finalizing spinner
