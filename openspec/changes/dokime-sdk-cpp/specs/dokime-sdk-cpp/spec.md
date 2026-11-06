## ADDED Requirements

### Requirement: C++ Test Authoring
Developers SHALL write Dokime suites in C++ using the shared concept names.

#### Scenario: End to End
- **WHEN** a C++ plugin is bundled, installed and run from the desktop
- **THEN** its steps, streams, prompts, criteria and report behave like a Rust plugin's

### Requirement: Simple Installation
Developers SHALL install the C++ SDK with CMake (`FetchContent` or a package manager), the language's idiomatic modern toolchain, and any
required command-line tools through mise.

#### Scenario: New Project
- **WHEN** a developer follows the install section in a new project
- **THEN** a few documented commands reach a building sample, with no manual downloads
