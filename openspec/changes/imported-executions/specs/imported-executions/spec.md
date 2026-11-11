## ADDED Requirements

### Requirement: Staged Manifest Admission

The host SHALL stage bounded, checksummed, resumable chunks under host-owned limits and backpressure, then atomically publish a validated manifest as an open import draft. It SHALL NOT use live stream admission or expose partially accepted evidence.

#### Scenario: Incomplete Upload

- **WHEN** upload stops before manifest validation
- **THEN** no draft or cycle member is visible and only resumable staging remains

#### Scenario: Validated Manifest

- **WHEN** all manifest chunks, schemas and checksums validate
- **THEN** one complete open draft is committed and its revision-bound handle is returned

#### Scenario: Transfer Limits

- **WHEN** a transfer exceeds a host limit or capacity
- **THEN** a typed limit error or backpressure applies without partial publication

### Requirement: Draft Visibility and Frontiers

An import draft SHALL be accessible only through its handle, absent from other clients, cycle membership, history, search and ordinary queries/exports until seal. Handle-scoped reads and follows SHALL bind a fixed revision and end at its committed evidence frontier.

#### Scenario: Draft Discovery

- **WHEN** another client lists cycles, history, search or exports before seal
- **THEN** the draft and provisional membership are absent

#### Scenario: Finite Draft Follow

- **WHEN** a handle follows committed draft evidence and replacement later succeeds
- **THEN** the follow ends at its original revision frontier without switching to replacement data

### Requirement: Preview Without Persistence

preview(document) SHALL validate and render a pinned draft revision without durable writes, report publication or draft mutation. Repeated documents SHALL be allowed. Temporary outputs SHALL be released on completion or cancellation; preview failure SHALL retain the draft.

#### Scenario: Repeated Preview

- **WHEN** a holder previews two different documents on the same revision
- **THEN** each returns a report without storing a document/report or changing the draft

#### Scenario: Preview Failure

- **WHEN** rendering fails or is cancelled
- **THEN** temporary work is released and the draft remains available with no published report

### Requirement: Conditional Seal Publication

seal(document) SHALL require the expected draft revision, render the final document, close evidence and freeze its report, then atomically publish the sealed execution, report and cycle membership. Failed sealing SHALL preserve the draft and prior cycle contents; closed cycle admission SHALL NOT be bypassed.

#### Scenario: Successful Seal

- **WHEN** the revision, document and cycle admission validate and rendering succeeds
- **THEN** the frozen execution/report and membership become visible together

#### Scenario: Failed Seal

- **WHEN** rendering, cycle admission or expected revision fails
- **THEN** nothing is published and prior cycle contents and the draft remain intact

### Requirement: Import Keys and Retry Receipts

Import keys SHALL be scoped to store identity and differ from mutation request identity. Successful mutations SHALL retain durable receipts; identical retries SHALL return the original result after restart or seal. Changed payload under one request identity SHALL conflict.

#### Scenario: Lost Seal Response

- **WHEN** a successful seal response is lost and its identical request is retried
- **THEN** the durable receipt returns the original execution rather than a sealed-key error

#### Scenario: Changed Retry Payload

- **WHEN** a request identity is reused for different content
- **THEN** the host returns an explicit conflict without applying the mutation

### Requirement: Conditional Replacement

A new request using an unsealed key SHALL replace its draft only at the expected revision. A new request using a sealed key SHALL be refused naming its execution. Creation, replacement, seal and discard SHALL prevent accidental concurrent overwrites through explicit conflicts.

#### Scenario: Stale Replacement

- **WHEN** two clients replace the same draft revision
- **THEN** at most one succeeds and the stale mutation conflicts without overwriting it

#### Scenario: Sealed Key

- **WHEN** a new request reuses a sealed import key
- **THEN** the host refuses it and names the existing execution

#### Scenario: Competing Creation

- **WHEN** two requests create the same store-scoped key expecting absence
- **THEN** at most one creates the draft and the other receives a conflict

### Requirement: Draft Recovery and Reclamation

The host SHALL own unsealed-draft expiry and SHALL NOT expire a revision during an active preview. Drafts SHALL be excluded from interrupted-run recovery. Discard, expiry and orphan cleanup SHALL reclaim independently owned draft columns, attachments, projections and metadata separately from logical deletion.

#### Scenario: Restart With Draft

- **WHEN** the host restarts with a committed import draft
- **THEN** it retains draft identity/receipts and does not close it as an interrupted run

#### Scenario: Expiry During Preview

- **WHEN** the expiry period elapses during a pinned preview
- **THEN** expiry cannot remove the active revision

#### Scenario: Discarded Storage

- **WHEN** conditional discard succeeds
- **THEN** the draft disappears and all its independently owned storage is reclaimed without promising shared database-file shrinkage

### Requirement: Revision-Bound Handles

Handles and previews SHALL bind store identity, execution and draft revision. Replacement, seal and discard SHALL require expected revisions. Replacement SHALL NOT change a preview snapshot, and an old preview SHALL NOT seal a newer draft implicitly.

#### Scenario: Replace During Preview

- **WHEN** a revision is replaced while its preview renders
- **THEN** the preview keeps its fixed snapshot and a stale seal conflicts

#### Scenario: Fresh Root

- **WHEN** a handle from another store identity is used
- **THEN** it is refused and keys in the new store form a separate namespace

### Requirement: Absolute Source Time

Each sample SHALL retain UTC source time, host receipt time and durable position; each stream SHALL declare its axis. Imports SHALL supply either absolute timestamps or UTC start plus exactly one elapsed_s/elapsed_ms/elapsed_us/elapsed_ns array. The wire SHALL carry integer nanoseconds; absent, ambiguous or unrepresentable clocks SHALL be refused.

#### Scenario: Relative Measurements

- **WHEN** a stream supplies UTC start and elapsed_ms with matching row lengths
- **THEN** source instants are resolved to UTC with exact integer-nanosecond wire values

#### Scenario: Unmapped Clock

- **WHEN** a stream has neither absolute timestamps nor UTC start and one elapsed array
- **THEN** admission refuses it rather than inventing measurement time

#### Scenario: Invalid Time Shape

- **WHEN** a stream supplies both forms, multiple elapsed arrays, wrong lengths or unrepresentable nanoseconds
- **THEN** admission returns a typed validation error without publication

### Requirement: Import-Time History

History SHALL place and time-filter an imported execution by its import time and separately show its measurement period as source-time coverage. Upload/import time SHALL NOT replace source time in plots or evidence queries.

#### Scenario: Historical Measurements

- **WHEN** sealed measurements predate their import by years
- **THEN** history places the execution at import time and shows the historical source range separately

### Requirement: Shared Columnar Wire

Imported stream tables SHALL use the plot pipeline columnar-frame dokime.v1 message with typed column arrays, explicit null validity, row count and timestamps. Signed/unsigned 64-bit values and bytes SHALL be preserved; Arrow/dataframe SDK layers SHALL be optional.

#### Scenario: Portable Batch

- **WHEN** each SDK transfers nulls, bytes and signed/unsigned 64-bit extremes
- **THEN** the same typed wire message preserves values and validity without a dataframe dependency

#### Scenario: Malformed Batch

- **WHEN** array lengths, validity or timestamps disagree with row count
- **THEN** manifest validation refuses the batch

### Requirement: Frozen Import Provenance

Evidence exports SHALL include origin, claimed importer and SDK version, source inventory/hashes, script-supplied processing description and declared outcome. The host SHALL hash uploaded bytes and verify checksums; hashes for unattached sources SHALL remain marked importer claims.

#### Scenario: Evidence Exported

- **WHEN** a sealed import is exported
- **THEN** structured provenance accompanies evidence and distinguishes host-verified upload hashes from unattached-source claims

### Requirement: Import Traceability and Trust

Imports SHALL permit explicitly marked importer-supplied traceability and SHALL reject step/attempt references and criteria. Execution and cycle reports SHALL explain that an import seal attests persistence, not measurement authenticity, and distinguish declared outcomes from Dokime evaluation.

#### Scenario: Importer Declaration

- **WHEN** an importer declares Passed and supplies device traceability
- **THEN** reports attribute both to the importer and state the persistence-only seal meaning

#### Scenario: Run Reference Supplied

- **WHEN** an import supplies criteria or a step/attempt reference
- **THEN** admission refuses it without manufacturing a TestCase

### Requirement: Shared Client Trust

Any connected client SHALL be permitted to call import operations under the shared trust model. Store-scoped keys, revision checks and explicit conflicts SHALL prevent accidental collisions; labels and handles SHALL NOT imply per-user ownership. Per-user authentication/ownership is out of scope.

#### Scenario: Another Connected Client

- **WHEN** another client has a valid handle and expected revision
- **THEN** the same operation policy applies without a fabricated importer ACL

### Requirement: Whole Imports and Corrections

Imports SHALL be whole submissions. A correction SHALL create a new execution linked to its predecessor rather than mutate sealed evidence; a sealed predecessor requires a new key. Incremental imports SHALL be out of scope.

#### Scenario: Correct Sealed Measurements

- **WHEN** a script submits corrected evidence for a sealed execution
- **THEN** a new key creates a linked execution and leaves the predecessor immutable

### Requirement: Flat SDK Import Flow

Rust, Python, C# and C++ SDKs SHALL offer flat Import composition and import_execution-equivalent admission returning an unsealed draft, repeatable preview(document), conditional seal(document), discard() and downloadable final report in their language idiom. No block construct SHALL be required.

#### Scenario: Script Iterates on Report

- **WHEN** a script attaches sources, supplies stream arrays, imports and previews two documents
- **THEN** it can seal the chosen document and save execution.report through explicit calls
