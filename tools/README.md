# Repository Tooling

Mise is the public entry point. Python modules in `python/src/dokime_tools/`
own environment setup, synchronization, version metadata, binding generation,
frontend preparation, desktop process supervision, native packaging, CI and
test execution. The shared Python environment lives at repository-root `.venv/`.

The desktop GUI owns its web tooling under `crates/dokime-gui/`: Biome, the
pnpm lock and the RPC generator and screenshot capture in its `tools/`. Rust
performance workloads live under the owning crate's `benches/`.

Each Python package carries its own `[tool.ty]` configuration. `lint:python`
type-checks the repository tooling with `ty check --project tools/python`;
`test:generated:python` type-checks the client package after the build has
verified its generated modules.
