# Native Desktop Performance Specification

## Purpose

Keep native desktop verification observable and bounded while preserving scene
assertions and native delivery checks for every owned execution report.

## Requirements

### Requirement: Scene-Owned Reports

Each desktop scene SHALL explicitly declare its owned execution reports and
finalize only those, preserving native save, artifact and browser reopen checks.

#### Scenario: Later Scene Completes

- **WHEN** a later scene finishes after earlier reports were saved
- **THEN** only its declared reports are exported and reopened

### Requirement: Observable Native Runtime

The harness SHALL log scene and step durations and attribute launches, fixture
preparation, execution waits, polling, report delivery and restarts without
asserting elapsed time or removing existing scene assertions.

#### Scenario: Full Native Lane

- **WHEN** both native stories run against prepared artifacts
- **THEN** timings identify runtime costs and every existing behavior is checked
