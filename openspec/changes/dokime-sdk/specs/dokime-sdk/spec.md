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

#### Scenario: Default Language
- **WHEN** a developer runs `dokime new my-plugin`
- **THEN** it creates a plain Rust project that builds with `cargo build`

#### Scenario: Minimal Tools
- **WHEN** a project is created for a language
- **THEN** its `mise.toml` pins only the tools that language needs

#### Scenario: Unsupported Language
- **WHEN** `--lang` names a language with no SDK
- **THEN** the command fails, lists the supported languages and creates nothing

### Requirement: One Command-Line Tool
Plugin authoring, installer packaging and binding generation SHALL live in the `dokime` CLI, and
`cargo-dokime` SHALL no longer exist.

#### Scenario: Removed Tool
- **WHEN** the change completes
- **THEN** no project file, template, workflow or document refers to `cargo dokime`

#### Scenario: Any Language Bundles Alike
- **WHEN** a Rust, C++ or Go project has been built with its own toolchain
- **THEN** `dokime plugin package` bundles it from the same collected-build input

### Requirement: Pure Starters
Each starter SHALL be an ordinary project in its language with a mise task wrapping the build and
the shared packaging step.

#### Scenario: Plain Build
- **WHEN** a developer runs the language's normal build command in a new starter
- **THEN** it builds without any Dokime command

#### Scenario: Bundle
- **WHEN** the developer runs the starter's bundle task
- **THEN** a `.dokime` bundle is produced
