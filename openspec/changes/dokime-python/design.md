# Design

API shapes remain proposed. The foundation decisions below describe this slice's source; later wrappers remain open.

## Flow

```python
import asyncio, dokime
from plugin import lab
async def main():
    async with dokime.Backend(port=50051, plugins=[lab]) as backend:
        backend.export_pairing("lab.dokime-remote", label="Bench 3")
        client = backend.client()
        cycle = await client.start_cycle("lab.capture", parameters={"rows": 1000})
        await cycle.wait()
asyncio.run(main())
```

## Layout

```text
sdk/python/
  Cargo.toml  pyproject.toml              # maturin, private _native, abi3
  src/{client,authoring,domain,runtime}/    # PySomething, registration in mod.rs
  python/dokime/                          # public modules, py.typed, hand-written .pyi
  tests/{unit,integration,acceptance,system}/
  examples/
```

Rust modules mirror public Python modules; stubs live at public paths, never private Rust paths.

## Reuse Of Landed Code

| Need | Landed | Change |
| --- | --- | --- |
| Backend | `dokime_server::host::Session::start` | Accept embedded providers |
| Pairing | `create_pairing` in the gRPC remote controller | Expose it; start listener |
| Provider | `runtime/embedded.rs::EmbeddedProvider` | Add operations, observers, sections |
| Client | `crates/dokime-client` | Wrap instead of Python wire code |
| Exclusive root | `local.lock`, `storage::Error::Owned` | Map to a Python exception |

Embedded templates already use host-level `dokime.template.*` operations. Extend this provider instead of adding another path.

## Rust To Python Calls

Tokio owns the runtime; Rust releases the interpreter while waiting.

- Async callbacks run on the loop that started the backend; synchronous callbacks run on worker threads.
- Existing operation deadlines remain authoritative. Cancellation cancels the Python task; ignored cancellation is reported.
- Exceptions become operation failures with tracebacks, never panics.
- Callback failures during shutdown are logged and cannot start new work.

## Fast Streams

`push` serves low-rate data. One batch crossing accepts NumPy arrays or Arrow columns, read with the interpreter released:

```python
await ctx.stream(Measurement).into(Samples).push_columns({"timestamp": timestamps, "voltage_v": voltages})
```

Fast Stream Support owns and measures throughput targets separately.

## Macro Mapping

| Rust | Python | Why It Differs |
| --- | --- | --- |
| `#[dokime::parameters]`, `#[parameter(...)]` | `@dokime.parameters`, `dokime.parameter(...)` | |
| `#[dokime::template(...)]` on `impl` | `@dokime.template(...)` on a class | |
| `step`, `observer`, `criterion`, `prompt`, `record`, `stream`, `section`, `suite`, `operation` attributes | Same decorator names and arguments | |
| `#[dokime::catalog] enum` | Lists | No enum dispatch |
| `#[dokime::message]`, `#[dokime::harmos]` | Dropped or dataclass | Binding owns serialization |
| `ctx.stream::<T>()` | `ctx.stream(T)` | Python syntax |
| `Result<(), Error>` | Raised exceptions | Python convention |

## Storage Format Identity

Clean-break development keeps v1. The identity must not depend on manually bumping a constant.

- Derive a build-time digest from declared stored schemas, starting with Harmos schema graphs and SQLite layout.
- Record the digest and writer commit before the first evidence write.
- A different digest or a populated root without a marker refuses with `StorageFormatMismatch`, naming root and both builds.
- Refusal rewrites, migrates and deletes nothing.

The coverage audit found JSON state snapshots, journals, settings, history and previews, custom SQLite object frames, and Arrow IPC outside the message graphs. Graphs alone cannot satisfy task 1.1.

Owner decision (2026-11-06): include normalized codec source, removing comments and whitespace between tokens while preserving literal contents and token boundaries. An edit that leaves stored bytes unchanged can occasionally refuse a root.

The explicit inputs in `crates/dokime-runtime/build/mod.rs` and `crates/dokime-domain/src/storage_format.rs` cover the SQLite layout and stored JSON, object-frame and Arrow IPC codecs, including native execution-context and report-publication frames, persisted query contract cuts, and transitive snapshot and journal types; unrelated source files are excluded. The build also hashes the Harmos graphs for application settings and the two stored history summaries. The writer commit is recorded separately and never contributes to format identity.

## Foundation Decisions

- `sdk/python` and its `dokime` plus `dokime-server` dependencies are owner-locked exceptions to the bindings skill. Native policy declares the source root and binding mapping.
- The provisional local distribution is `dokime-python`, importing `dokime`. `Private :: Do Not Upload` and the release publisher refuse registry upload until naming is settled.
- Maturin builds one `cp313-abi3` wheel per native platform. Cargo defaults and Product/Tests/Bench builds exclude the explicit Python producer.
- Product precedes Python in CI. Both use the normal Cargo target tree and profile; the wheel enables the server's media feature for release. Different resolved feature sets can still produce separate Cargo units.
- The updated workstation guidance removes the build lock; Python retains the existing machine/cache wrapper and compiler-output preflight.
- Package copies native wheels without compilation. CI, simulation and release inputs use native paths; releases collect both wheels. No pure wheel or source distribution remains in this producer.
- Windows dev measurement: wheel 227,830 bytes; wheel task 42.97 s, including 19.35 s Clippy and 11.59 s native compilation after the server-library baseline; all 516 pre-existing Rust archives retained their size and modification time, and three PyO3 archives were added. This slice's import surface is version metadata only. Its wheel size does not predict the completed client/backend SDK size. Client code, tests and wire generation remain owned by tasks 2.1–2.5.

## Inherited Mechanisms

The gRPC service, Harmos sidecar lifecycle, `local.lock`, reports and Python-client behaviour remain. The complete change replaces `packages/dokime-client`, generated wire code and descriptors; the pure `Build / Product / Python (any)` producer is removed in this slice.
