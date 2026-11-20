## ADDED Requirements

### Requirement: Harmonized Reference Plugins

Each language SDK (Rust, Python, C#, C++) SHALL provide one small reference plugin that declares the same operations, observers and streams as the other three, including an echo operation, and the benches SHALL measure these reference plugins and never a starter project.

#### Scenario: Same Declarations in Every Language

- **WHEN** the reference plugins of all four languages are admitted by a host
- **THEN** their declared operations, templates, streams, observer intervals and row schemas are identical

#### Scenario: Starter Stays Out of the Benches

- **WHEN** a starter project changes its content for new authors
- **THEN** no bench case and no baseline changes

### Requirement: One Driver With a Catalog Guard

One driver SHALL run the identical scenario against each language's reference plugin and record the same measures, and it SHALL refuse to measure a plugin whose declared catalog differs from the others.

#### Scenario: Identical Scenario per Language

- **WHEN** the driver measures the C# reference plugin and then the Python reference plugin
- **THEN** both runs execute the same steps with the same inputs and produce the same set of measurement names

#### Scenario: Catalog Differs

- **WHEN** one reference plugin declares a stream, operation or observer interval that the others do not
- **THEN** the driver records no measurement for that plugin and fails with the first differing identifier and the languages involved

### Requirement: SDK Measures

The driver SHALL record, for each language and platform, the operation round trip as a latency distribution and as calls per second for the echo operation, the stream throughput in samples per second from an observer including 128 or more streams at up to 1 kHz each and the ceiling of a single stream, the time from process start until the host admits the plugin, and the idle and loaded memory of the plugin process, with units in square brackets in every label.

#### Scenario: Round Trip

- **WHEN** the driver calls the echo operation of a reference plugin
- **THEN** it records the latency percentiles and the calls per second for that language and platform

#### Scenario: Throughput at Target

- **WHEN** the driver runs 128 streams at 1 kHz each on a reference plugin
- **THEN** it records the samples per second the host accepted, labelled `Throughput [samples/s]`, whether or not the plugin sustained the declared rate

#### Scenario: Start to Ready and Memory

- **WHEN** the driver starts a reference plugin and then loads it
- **THEN** it records the time until admission and the plugin process's memory before and during load

#### Scenario: Stage Not Measurable

- **WHEN** a measure cannot be taken on a platform
- **THEN** the report records that measure as not measured with its reason instead of omitting it

### Requirement: Measurements Never Gate

SDK, protocol and storage benches SHALL be measurements compared with regression baselines kept per language and per platform, SHALL never pass or fail a test or gate Nightly publication, and the cross-language comparison SHALL be a report without thresholds.

#### Scenario: Regression in One Language

- **WHEN** a bench case is slower than its baseline beyond the configured threshold
- **THEN** the Nightly report lists the regression and the Nightly release is still published

#### Scenario: Baseline per Language and Platform

- **WHEN** the Python bench runs on Windows
- **THEN** its results are compared only with the previous Python results on Windows from `dev`

#### Scenario: Cross-Language Report

- **WHEN** a Nightly run completes the SDK benches
- **THEN** the report shows each measure side by side for every language and platform without a pass or fail verdict

#### Scenario: No Wall-Clock Test

- **WHEN** a test exercises a reference plugin
- **THEN** its result does not depend on elapsed time, and timing exists only in the benches

### Requirement: Fixture Reuse

The C# and C++ SDK benches SHALL launch reference plugins built by the existing Nightly fixture builds and SHALL NOT build SDK dependencies themselves.

#### Scenario: C++ Bench Job

- **WHEN** `Bench / SDK / C++` runs
- **THEN** it uses the archived C++ fixture build and spends no time building C++ dependencies

### Requirement: Product Protocol and Storage Benches

`Bench / Protocol` SHALL measure encode and decode cost and encoded size of this product's evidence, sample, plot and report messages, and `Bench / Storage` SHALL measure this product's objects only through the library's storage operations; transport and storage-engine benches SHALL remain the library's.

#### Scenario: Protocol Message Cost

- **WHEN** `Bench / Protocol` runs
- **THEN** it records encode time, decode time and size in bytes for each measured message at fixed sizes

#### Scenario: Storage Through Library Operations

- **WHEN** `Bench / Storage` runs
- **THEN** it writes an execution, seals it and queries one million samples through the library's storage operations and records each cost

#### Scenario: Storage Bench Before the Cutover

- **WHEN** the product still persists through its own store
- **THEN** no `Bench / Storage` job exists

### Requirement: Bench Job Names

Bench jobs SHALL be named by Stage / Component / Variant as `Bench / SDK / <Language>`, `Bench / Protocol` and `Bench / Storage`, with the platform in parentheses, next to the existing `Bench / Desktop / <Variant>` jobs.

#### Scenario: Nightly Job List

- **WHEN** a maintainer reads the Nightly jobs
- **THEN** the SDK benches appear as `Bench / SDK / Rust`, `Bench / SDK / Python`, `Bench / SDK / C#` and `Bench / SDK / C++` for each platform
