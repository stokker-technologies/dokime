# Repository Tooling

Mise is the public entry point. Python modules in `python/src/dokime_tools/`
own environment setup, synchronization, version metadata, binding generation,
frontend preparation, desktop process supervision, native packaging, CI and
test execution. The shared Python environment lives at repository-root `.venv/`.

`node/gui/` retains only the RPC generator and screenshot capture until their
owning migrations land. Native frontend interoperability drivers live with the
GUI tests. Rust performance workloads live under the owning crate's `benches/`.

The pinned Ruff and Pyright versions check both product and tooling sources
through the repository `format`, `lint:fix` and `check` tasks.
