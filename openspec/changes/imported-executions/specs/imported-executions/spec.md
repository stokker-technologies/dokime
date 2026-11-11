## ADDED Requirements

### Requirement: Ordinary Execution With Import Origin

Dokime SHALL store an import as an ordinary execution in the named cycle, creating the cycle if absent. It SHALL have Import origin with source hashes, importing client and import time, parameters and streams, and no steps or criteria. PluginRun origin SHALL own plugin, template, digest and plugin start time instead of TestCase.

#### Scenario: Import Stored

- **WHEN** a client imports measurements into a named cycle
- **THEN** the ordinary execution store holds the execution with Import origin and the cycle's membership
- **AND** existing history, statistics, document and report queries can read its evidence

#### Scenario: Plugin Provenance

- **WHEN** the host creates a plugin execution
- **THEN** its PluginRun origin holds plugin, template, materialization digest and process start time and those fields are absent from TestCase

### Requirement: Imported Provenance on Every Surface

History, plots, documents, reports and seals SHALL mark imported executions as imported and retain their origin. Original source files MAY be attached as artifacts; attached bytes SHALL match the origin's hashes.

#### Scenario: Evidence Reviewed

- **WHEN** a user reviews an imported execution in history, a plot, its document, report or seal
- **THEN** each surface identifies it as imported with accessible source provenance

#### Scenario: Original File Attached

- **WHEN** an import attaches a source file
- **THEN** its artifact bytes are verified against the source hash retained in the origin

### Requirement: Declared Outcome Attribution

An importer MAY declare passed or failed. Dokime SHALL store and show that outcome as declared by the importer, never as evaluated by Dokime. An absent declaration SHALL NOT imply passed. Imported executions SHALL NOT accept criteria; plugin operations remain the only criteria mechanism.

#### Scenario: Importer Declares Passed

- **WHEN** an importer declares passed
- **THEN** execution views and the report show passed as declared by the importer without a Dokime criterion result

#### Scenario: No Declaration or Invalid Criteria

- **WHEN** the import omits an outcome or supplies criteria
- **THEN** omission does not produce a passed result and supplying criteria is refused

### Requirement: Atomic Import and Report

One import call SHALL upload, store, seal and generate the report atomically. Any failure SHALL leave no import execution, new cycle, membership change, artifacts, report or consumed key. Existing cycle contents SHALL remain intact. Recovery SHALL remove abandoned staging.

#### Scenario: Report Generation Fails

- **WHEN** report generation fails after stream and artifact staging
- **THEN** the call fails and none of the import's state remains in the store or report files

#### Scenario: Cancelled or Interrupted Import

- **WHEN** cancellation or a host crash interrupts an import before commit
- **THEN** rollback or recovery removes staged effects, preserves existing cycle contents and releases the key

### Requirement: Sealed Key Refusal

The caller SHALL supply an import key. Reusing a key whose execution is sealed SHALL fail with an error naming that execution. Key reservation and publication SHALL serialize competing uses within the store; a rolled-back import SHALL NOT consume its key.

#### Scenario: Retry After Lost Response

- **WHEN** the client repeats a key after the first import sealed successfully
- **THEN** the call fails naming the existing execution and creates no duplicate

#### Scenario: Concurrent Key Use

- **WHEN** two imports use the same key concurrently
- **THEN** at most one publishes and the other cannot create a duplicate execution

### Requirement: Columnar Source Time Axis

Imported streams SHALL transfer as one columnar table per stream, not one message per sample. Their source timestamps SHALL remain the time axis for storage, plots, statistics and document queries, independent of upload and import time.

#### Scenario: Historical Measurements

- **WHEN** a script imports a table whose timestamps predate the upload
- **THEN** plots and time-range queries use the source timestamps and statistics read the same stored columns

### Requirement: Composed Import in Every SDK

Rust, Python, C# and C++ client SDKs SHALL compose an import value and submit it through one import_execution-equivalent call, returning the sealed execution and downloadable report. Python SHALL require no context manager; explicit resource close SHALL be available.

#### Scenario: Script Downloads a Report

- **WHEN** a script composes files, stream tables, records, document and declared outcome and submits the value
- **THEN** one call returns the sealed ordinary execution and the script can save its generated report
