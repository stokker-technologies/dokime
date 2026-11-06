## ADDED Requirements

### Requirement: Runtime-Free Authoring
A plugin SHALL build against `dokime-sdk` without compiling `dokime-runtime`.

#### Scenario: Plugin Dependency Tree
- **WHEN** a generated starter plugin is resolved
- **THEN** its dependency tree does not include `dokime-runtime`

#### Scenario: Existing Plugins
- **WHEN** the host and OBC plugins build against the SDK
- **THEN** their behaviour and contract tests are unchanged

### Requirement: Shared Concept Names
Every language's SDK SHALL use the same names for suites, templates, steps, observers, criteria,
prompts, records and streams.

#### Scenario: Comparing SDKs
- **WHEN** a concept exists in two SDKs
- **THEN** it has the same name, or a recorded reason for a language-specific difference

### Requirement: Simple Installation
Developers SHALL install the Rust SDK with `cargo add`, the language's idiomatic modern toolchain, and any
required command-line tools through mise.

#### Scenario: New Project
- **WHEN** a developer follows the install section in a new project
- **THEN** a few documented commands reach a building sample, with no manual downloads
