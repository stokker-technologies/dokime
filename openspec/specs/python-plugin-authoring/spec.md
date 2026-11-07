# Python Plugin Authoring Specification

## Purpose

Define native admission and execution of Python declarations, callbacks and column batches through the embedded provider.

## Requirements

### Requirement: Decorator API
Python SHALL define suites, templates, parameters, steps, observers, criteria, prompts,
records, streams, sections and operations with decorators harmonized with the Rust macros.

#### Scenario: Equivalent Definition
- **WHEN** a Python template declares the same identifiers, parameters and streams as a
  Rust template
- **THEN** the catalog, validation and document behaviour match

#### Scenario: Invalid Declaration
- **WHEN** a declaration is invalid, such as an undeclared stream or a bad identifier
- **THEN** registration raises a typed error naming the declaration, before any execution

### Requirement: Rust Calls Python
Rust SHALL run Python plugin callbacks through an embedded provider that supports
operations, observers and settings sections.

#### Scenario: Async Step
- **WHEN** an `async def` step runs
- **THEN** it runs on the event loop that started the backend

#### Scenario: Blocking Step
- **WHEN** a plain `def` step blocks
- **THEN** it runs on a worker thread, and the event loop and backend stay responsive

#### Scenario: Python Exception
- **WHEN** a callback raises
- **THEN** that operation fails with the traceback in its failure details, and the backend
  keeps running

### Requirement: Authoritative Deadlines
Python callbacks SHALL obey the same operation policies as Rust plugins.

#### Scenario: Deadline Reached
- **WHEN** an operation's deadline passes
- **THEN** its Python task is cancelled and the operation reports the deadline failure

#### Scenario: Ignored Cancellation
- **WHEN** a callback ignores cancellation
- **THEN** the failure names the callback, and shutdown still completes within its policy

### Requirement: Batch Stream Publication
Python SHALL publish stream rows in batches from column buffers in one host crossing.

#### Scenario: Column Batch
- **WHEN** a step publishes NumPy or Arrow columns of equal length
- **THEN** every row is stored in order, as if pushed one at a time

#### Scenario: Unequal Columns
- **WHEN** the columns differ in length
- **THEN** the call raises a typed error and publishes nothing

### Requirement: Plugin Identity Conflict
A Python plugin SHALL NOT use the ID of an installed plugin.

#### Scenario: Same ID as Installed
- **WHEN** a script registers a plugin whose ID matches an installed plugin
- **THEN** startup raises a typed error naming the ID, and no backend remains
