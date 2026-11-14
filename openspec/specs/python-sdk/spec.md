# Python SDK Specification

## Purpose

Define Dokime's native Python interface, typed exports, client parity, platform wheels and end-to-end proof.

## Requirements

### Requirement: Rust API Mirror
The public Python API SHALL mirror the Rust API's concepts, names and arguments,
deviating only where Python requires it.

#### Scenario: Documented Deviation
- **WHEN** a Python name or shape differs from its Rust counterpart
- **THEN** the design's mapping table records the difference and its reason

### Requirement: Simple Installation
Developers SHALL install the package with `uv add`, the language's idiomatic modern toolchain, and any
required command-line tools through mise.

#### Scenario: New Project
- **WHEN** a developer follows the install section in a new project
- **THEN** a few documented commands reach a building sample, with no manual downloads
