# Project Guidance

## Development Storage Policy

- Dokime is in development. Database schemas and evidence formats may change without version bumps for now; keep their baseline at v1.
- Clean database breaks are allowed. Do not add migrations or legacy compatibility solely to preserve development data. Use fresh storage roots when a schema change makes existing data incompatible; never silently delete or reset existing data.

## Documentation

- Start at `README.md` and `docs/product-guide.md`; its Documentation section links the topic guides in `docs/`. Read `docs/gui.md` before changing `crates/dokime-gui/`.
- Keep documentation in `docs/` and refer to code by path. Do not add `AGENTS.md`, `CLAUDE.md` or other Markdown notes inside code directories; package READMEs that registries publish are the exception.
- Follow the plugin operation boundaries and step form in [Plugin Authoring](.agents/skills/dokime-authoring/references/plugins.md) when authoring plugins.

## Python SDK

- `sdk/python/` owns the native PyO3 SDK, public `python/dokime/` modules and hand-written stubs, layered tests, benchmarks and examples. `dokime` and `dokime-server` are its owner-locked native dependencies; it replaces the former generated Python client.
- Build its wheel explicitly with `mise run build:python`; ordinary Rust builds exclude it. Use the installed wheel for Python proof and `check:python:bindings` for strict stubtest.
- `sdk/python/examples/starter/` is the single source embedded by `dokime new --lang python`; update its operations and suites together with the CLI and system story. Keep driver imports inside operations and use `ctx.operation(operation)` from steps and observers.
- `test:python:system` runs the generated-project, pairing, prompt, capture and restart story on Windows and Linux in Nightly only. Focused local fixture builds use `build:python:fixtures`; `bench:python:system` measures bridge preparation separately from admitted storage publication.
