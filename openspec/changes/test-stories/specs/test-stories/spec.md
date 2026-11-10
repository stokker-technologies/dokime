## ADDED Requirements

### Requirement: Storyline Tests

A test at integration level and above SHALL follow one user journey in one file and read top to bottom as the user's actions and what they then see, with mechanics provided by shared modules.

#### Scenario: Reading a Story

- **WHEN** a maintainer opens a test at integration level or above
- **THEN** each stage is named by a short comment and every line is an action or a check, with no scenario or assertion hidden in a helper

#### Scenario: Shared Mechanics

- **WHEN** a test needs a running backend, a connected client, a plugin, or to wait for an event
- **THEN** it uses the shared module for it and does not rebuild it

### Requirement: Deterministic Time in Tests

The result of a test SHALL NOT depend on real elapsed time or on the wall clock.

#### Scenario: Waiting Inside a Plugin

- **WHEN** a story waits inside a plugin step
- **THEN** the host's virtual clock advances and the story continues without real time passing

#### Scenario: Fixture Timestamps

- **WHEN** a test needs a timestamp for a record, sample or receipt
- **THEN** it uses a fixed value

#### Scenario: Hang Guard

- **WHEN** a test bounds how long it may wait for a real process or browser
- **THEN** the bound only stops a hung test and no assertion depends on the time taken

### Requirement: Workloads Belong to Benchmarks

A correctness test SHALL use the smallest input that crosses the boundary it proves, and larger workloads SHALL run as benchmarks.

#### Scenario: Large Transfer

- **WHEN** framing and digests are proven by a small multi-frame upload
- **THEN** a transfer of hundreds of megabytes exists only as a benchmark
