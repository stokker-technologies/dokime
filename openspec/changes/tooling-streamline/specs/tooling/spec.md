## ADDED Requirements

### Requirement: One Pinned Protocol Inventory

Generation SHALL resolve library protocols from the workspace Cargo library
revision into ignored build storage and SHALL NOT track a vendored copy.
All language generators and Buf SHALL consume those resolved library inputs.

#### Scenario: Fresh Clone Generation

- **WHEN** a developer generates without an existing protocol export
- **THEN** generation exports the pinned library's proto directory from git and stages current product protocols automatically

#### Scenario: Generated Drift

- **WHEN** a developer runs either generation check entry point
- **THEN** it resolves the same inputs and rejects missing, extra or changed generated output without rewriting it

#### Scenario: Pin Change Bootstrap

- **WHEN** a developer edits the workspace library revision and runs protocol generation once
- **THEN** one projection owner updates consumer manifests before refreshing locks and tooling and invoking the locked generator

### Requirement: Shared SDK Installation

The SDK install task SHALL reuse profile-aware wheel installation and the same
example-distribution preparation used by binding checks and test consumers.

#### Scenario: Selected Profile Installation

- **WHEN** SDK installation selects a build profile
- **THEN** the shared installer installs that profile's wheels and the example fixtures without replacing them with a hard-coded dev wheel

### Requirement: Compile-Only Focused Selection

Focused compile-only selections SHALL omit nextest's ignored empty-test policy.

#### Scenario: Compile Without Execution

- **WHEN** a focused selection includes --no-run
- **THEN** its command omits --no-tests while executed selections retain empty-test failure

### Requirement: Language-Agnostic Entry Points

A developer SHALL be able to prepare, generate, format, lint, check, build, test, benchmark and package the project through one task each, without knowing which languages it contains.

#### Scenario: Whole Project

- **WHEN** a developer runs the test task without arguments
- **THEN** it runs the tests of every language and layer

#### Scenario: Selection

- **WHEN** a developer passes a language or a layer to the test task
- **THEN** it runs only that selection

#### Scenario: No Orphan Tasks

- **WHEN** the task list is inspected
- **THEN** every task is called by a developer workflow, another task or CI

### Requirement: Machine Concerns Stay Out of Project Tooling

Project tasks and tooling SHALL NOT check or reclaim disk space, and incremental compilation SHALL be disabled only by a variable set in CI.

#### Scenario: Local Build

- **WHEN** a developer builds locally
- **THEN** no compiler output is scanned or deleted first, and incremental compilation is used

#### Scenario: CI Build

- **WHEN** CI builds
- **THEN** incremental compilation is disabled by one variable defined in the workflows

### Requirement: Workflows Call Tasks

A CI job SHALL run a project task for its work and SHALL NOT carry build, test or packaging logic of its own.

#### Scenario: Reproducing a Job

- **WHEN** a CI job fails
- **THEN** a developer reproduces it locally by running the task the job names with the same arguments

### Requirement: Self-Contained Checkout

Everything needed to prepare, build, test, package and release the project SHALL live in its own repository, so the tooling is this project's own copy and not a shared dependency.

#### Scenario: Fresh Clone

- **WHEN** a developer clones this repository alone and runs the prepare task
- **THEN** every entry point works, with no other repository checked out and no company tooling installed
