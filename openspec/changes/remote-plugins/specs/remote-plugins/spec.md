# Remote Plugins Spec Delta (Proposed)

## ADDED Requirements

### Requirement: Plugins Connect by URL

Dokime SHALL load a plugin by connecting to a URL the user adds in settings, and SHALL NOT install, store or start plugin processes.

#### Scenario: Plugin Added

- **WHEN** the user adds a plugin URL whose server is reachable
- **THEN** the entry shows ready with the plugin's id, version and catalog digest, and its templates and operations are available

#### Scenario: Plugin Unreachable

- **WHEN** the plugin's server is not reachable
- **THEN** the entry shows unavailable with the reason, and Dokime reconnects without user action when the server comes back

### Requirement: Dokime Catalog Sections

Dokime SHALL read suites, records, listeners, telemetry, analyses, settings and run-start contracts from versioned sections of the remote plugin's catalog.

#### Scenario: Template From a Remote Plugin

- **WHEN** an execution runs a template that a remote plugin's `dokime.suites` section declares
- **THEN** Dokime runs it through the plugin's `<template>.run` and `<template>.signal` operations, and the plugin's step code reaches Host-visibility operations as child calls

### Requirement: Versioned Automation Contract

Every release SHALL attach the Public product operation protos under the `dokime.automation.v1` package, and a breaking change SHALL move the package to a new version.

#### Scenario: Client Connects With a Pairing File

- **WHEN** a client built from the released protos connects with a `.dokime-remote` pairing file
- **THEN** it calls Public product operations without a Dokime client library
