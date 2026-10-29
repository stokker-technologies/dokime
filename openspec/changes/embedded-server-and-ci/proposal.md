# Embedded Server And CI

## Why

Complete the approved server composition simplification and CI repairs together. Native tests must not require an operator's desktop keyring. Default Cargo profiles and nightly-only starter consumer compilation bound ordinary CI cost.

## What Changes

- Replace the standalone server executable with `dokime server`; embed the existing server library in the GUI.
- Keep one library-owned server lifecycle, with process logging and signals owned by CLI or GUI.
- Inject credential storage at composition for deterministic tests; production continues to require OS secure storage.
- Restore Cargo defaults and use the built-in test profile for Tests and matching Clippy, preserving graph artifact namespaces.
- Consume the existing test archive for compiler-contract execution rather than rebuilding the GUI.
- Run expensive generated-starter consumer compilation only through an explicit nightly test job; retain cheap scaffold tests ordinarily.
- Complete the desktop lifecycle and backend-selection audit: retain shutdown ownership across retries, focus repeated launches per application root, and manage Local and saved remotes through the existing session owner and backend catalog.
- Remove obsolete GUI projections, route compatibility and remembered-parameter compatibility. Unsupported stored formats fail explicitly without silent deletion or migration.
- Align dependency features and TLS providers with their intended functionality, remove proven unused dependencies, and validate compatible dependency updates.

## Impact

CLI discovery, GUI lifecycle, packaging, installers, release assets, tutorials, process tests, build tooling, manifests and README migrate together. Remote attachment, authentication, storage lease safety and behavioral assertions remain supported. The user explicitly authorizes clean compatibility breaks; existing files are never silently reset or deleted. Hub supplies declarative pipeline selection and shared default-profile tooling. Hosted CI remains mandatory.
