# Dokime Python

## Why

Most test engineers write Python, and Rust-only authoring is Dokime's largest
adoption barrier. Python users also get a thinner product today: `dokime-client`
only controls a backend that something else started, and it maintains its own
generated wire layer beside the Rust client.

A PyO3 package gives Python the same position as the GUI and the CLI. A script can
start a backend, register suites written in Python and run them, while the GUI
attaches to the same backend through a pairing file.

## What Changes

- Add `sdk/python`, a thin PyO3 interface beside the GUI and the CLI.
- Replace `packages/dokime-client` and its generated wire code with that package.
- Mirror the Rust API, including a decorator API harmonized with the Rust macros.
- Start a backend from Python and export a pairing file for the GUI.
- Register plugins written in Python through an embedded provider; Rust calls their
  Python functions. These plugins live only as long as the backend that registered them.
- Extend `EmbeddedProvider` with the operations, observers and settings sections
  Python plugins need.
- Refuse a data root written by a different storage format, for every backend.
- Add a Python system test lane that runs in Nightly.
- Build one abi3 wheel per machine, after the product build.

## Locked Decisions

- **Layer:** logically the same level as the GUI and the CLI. It depends on `dokime`
  and `dokime-server`, a declared exception to the bindings' facade-only rule.
- **Data root:** the same folder as the GUI and the server. `local.lock` still allows
  one backend per root at a time.
- **Ephemeral plugins:** a Python plugin is absent from any backend that did not
  register it. Its evidence stays in the data root.
- **Installed plugins:** a Python-started backend always loads them, with no flag.
- **ID conflict:** a Python plugin using an installed plugin's ID is refused.
- **Storage format:** a backend refuses a root from another format and names the build
  that wrote it. This is a refusal only: no migration, compatibility or reset.
- **Stubs:** written by hand in the crate's `python/` folder, mirroring its modules.
- **Decorators:** the Rust macro names and arguments, deviating only when necessary.
- **Concepts:** authoring names follow the concept model shared by every language's SDK.
- **Name:** decided later. The `dokime` PyPI project is unrelated and also imports as
  `dokime`, so the package cannot be published until the name is settled.

## Non-Goals

- Packaging Python plugins as distributable `.dokime` bundles or sidecars.
- SDKs for other languages, including the parked shared protocol crate.
- Hot reload of a Python plugin: changing one means re-running the script.
- Migrating existing data roots.

## Impact

- **Build:** a new native wheel per platform, excluded from default builds. Wheel size
  and build-cache reuse are measured during implementation.
- **Consumers:** `dokime-client` users move to the new package. Its PyPI project gets a
  final release pointing there once the name is settled.
- **Dependency:** relies on the rule that completed executions and cycles, even
  unsealed, do not depend on the plugin that ran them. That change lands separately.
