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

### Requirement: Project Creation
`dokime new <name> --lang <language>` SHALL create a plugin project, defaulting to Rust and
supporting every language with an SDK, without needing a Rust toolchain for other languages.
`cargo dokime sidecar new` SHALL stay for Rust, harmonized with `cargo harmos sidecar new`, and
both commands SHALL share one implementation.

#### Scenario: Default Language
- **WHEN** a developer runs `dokime new my-plugin`
- **THEN** it creates the same Rust project as `cargo dokime sidecar new`

#### Scenario: Minimal Tools
- **WHEN** a project is created for a language
- **THEN** its `mise.toml` pins only the tools that language needs

#### Scenario: Unsupported Language
- **WHEN** `--lang` names a language with no SDK
- **THEN** the command fails, lists the supported languages and creates nothing
