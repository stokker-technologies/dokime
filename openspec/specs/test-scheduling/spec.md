# Test Scheduling Specification

## Purpose

Coordinate Dokime test lanes through isolated worker pools and shared prepared
inputs, preserving exclusive desktop ownership, native evidence and failure status.

## Requirements

### Requirement: Concurrent Test Entry
The test entry SHALL execute every existing lane concurrently after shared input
preparation, preserve all deadlines and failures, and retain complete lane logs,
exit statuses and time summaries. Five minutes is a soft target, never a deadline.

#### Scenario: Independent Lanes
- **WHEN** the owner runs `mise run test` with matching built artifacts
- **THEN** independent lanes overlap and the entry reports total and per-lane wall times
- **AND** every lane runs even if another fails, with a nonzero aggregate exit on failure

### Requirement: Isolated Parallel Workers
The runners SHALL parallelize isolated tests while preserving resource ownership.

#### Scenario: Shared Inputs And Exclusive Desktop
- **WHEN** concurrent lanes consume wheels, generated contracts and fixture artifacts
- **THEN** preparation finishes before readers start and no lane rewrites those inputs
- **AND** native desktop stories run sequentially because they share one desktop session

#### Scenario: Standalone Lanes
- **WHEN** an individual lane task is selected
- **THEN** it remains independently usable with its normal preparation and reports

### Requirement: Hosted Scheduling
Hosted test lanes SHALL depend only on their required producers and retain their
self-contained bootstrap without access to `.stkr/`.

#### Scenario: Producers Complete
- **WHEN** the matching artifact producers finish
- **THEN** independent hosted lanes can start without waiting for other test lanes
