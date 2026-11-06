# Design

Backend and authoring API shapes remain proposed. The foundation and client decisions below describe implemented source; their remaining wrappers stay open.

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

The coverage audit found JSON state snapshots, journals, settings, history and previews, custom SQLite object frames, and Arrow IPC outside the message graphs. Graphs alone cannot satisfy task 1.1. The owner decision on supplementing them with normalized codec inputs remains pending; no conservative-refusal exception is assumed.

## Foundation Decisions

- `sdk/python` and its `dokime` plus `dokime-server` dependencies are owner-locked exceptions to the bindings skill. Native policy declares the source root and binding mapping.
- The provisional local distribution is `dokime-python`, importing `dokime`. `Private :: Do Not Upload` and the release publisher refuse registry upload until naming is settled.
- Maturin builds one `cp313-abi3` wheel per native platform. Cargo defaults and Product/Tests/Bench builds exclude the explicit Python producer.
- Product precedes Python in CI. Both use the normal Cargo target tree and profile; the wheel enables the server's media feature for release. Different resolved feature sets can still produce separate Cargo units.
- The updated workstation guidance removes the build lock; Python retains the existing machine/cache wrapper and compiler-output preflight.
- Package copies native wheels without compilation. CI, simulation and release inputs use native paths; releases collect both wheels. No pure wheel or source distribution remains in this producer.
- Windows dev measurement: wheel 227,830 bytes; wheel task 42.97 s, including 19.35 s Clippy and 11.59 s native compilation after the server-library baseline; all 516 pre-existing Rust archives retained their size and modification time, and three PyO3 archives were added. This slice's import surface is version metadata only. Its wheel size does not predict the completed client/backend SDK size. Client code, tests and wire generation remain owned by tasks 2.1–2.5.

## Client Parity Decisions

- Rust `Client` owns the live connection, negotiation, identity checks, request deadlines and cancellation. PyO3 async futures release Python while waiting and propagate asyncio cancellation by dropping the Rust future. Closing the Python connection closes the shared Rust handle without stopping admitted backend work.
- Python retains its language-specific owned async contexts, lazy finite readers, immutable value views and optional dataframe conversion. These compose Rust client operations and retain the existing lifecycle and bounded-materialization contracts; no Python gRPC transport remains.
- The client identity snapshot stays at `dokime.client.Backend`; the root `dokime.Backend` name remains reserved for the backend async context in task 3.2.
- Private interpreter message classes are reflected from the Rust catalog through Harmos. No generated Python wire tree, descriptor asset, schema projection implementation or Python contract exporter remains. `dokime.client.contracts.operation(service, method)` replaces generated root operation constants; live plugin discovery still uses the authorized Rust catalog.
- A connection reuses reflected message classes for identical complete contracts and provider generations in its current catalog. Rediscovery still fetches Rust's authorized entries and refreshes grants/availability, while retired contracts leave the cache. This preserves request-class identity across rediscovery and avoids reconstructing every descriptor pool for each newly used method; no historical or unbounded reflection cache is added.
- Raw Rust client calls authorize external gRPC paths through that same live catalog; product methods retain the negotiated availability check. Streaming input uses a one-frame bounded channel, with explicit finish/cancel and final-status observation.
- The Rust raw-stream response accessor retains initial metadata for Python stream errors. Native causes and binary status details survive exception translation. Bridge-only mutex acquisition is interpreter-aware; native guards are released before Python conversion, and explicit stream close drains cancellation before returning.
- Error classifications and input refusals follow the Rust client; exception prose is not a compatibility contract. Socket fixtures now advertise both Backend and Runtime discovery methods and use their distinct response contracts, plus the authorized Host catalog.
- A malformed remote operation catalog raises `ProtocolError`; exceeding the Rust catalog budget raises `RpcError` with `RESOURCE_EXHAUSTED`. Neither is classified as a local argument error.
- Pairing and profile reading, validation, candidate selection and remembered-endpoint writes use the Rust implementations. Python profile objects are snapshots. Keyring-backed profiles retain the explicit pairing-file path required by the former Python client.
- Python checks an explicitly empty profile-path environment override before calling Rust: Windows removes an empty process environment variable while Python retains the assigned mapping entry. Profile selection and remembered-endpoint writes use the same resolved path.
- Foreign-operation socket fixtures are exported by a test-only Rust example through the authoritative schema and fingerprint constructors. Tests use valid differing contracts for drift and separately reject corrupted fingerprints; no Python digest implementation or validation exception is introduced.
- The standalone tutorial creates its lock after adding the matching native wheel. Its old platform-independent wheel lock and digest are removed rather than preserving a nonexistent artifact. Distribution naming remains provisional; no registry upload is enabled.
- Hand-maintained stubs mirror public modules and the private built extension. Built-wheel stubtest is the `check:python:bindings` task; generated-Python checks are retired. Removed Python transport retry/certificate internals and the settings codec round-trip test are already owned by Rust transport/domain tests; socket-level Python connection, cancellation, pairing and lifecycle tests remain.
- Stubtest uses pinned mypy 2.4.1 with Python 3.15 typing metadata support. The strict stubtest entry adapts its alias comparison to unwrap PEP 695 targets and compare equivalent multi-value Literal forms; it delegates every comparison to the original checker and preserves modern Python source conventions. Generic stubs declare their interpreter-provided type-parameter tuples, native handles are final, and constructor defaults match runtime; no stubtest allowlist or ignored errors are used.
- `build:python:fixtures` is the recurring focused producer for native Python acceptance: CLI, collector, host plugin and OBC, with no desktop or test archive. Its isolated fixture root can be selected with `test:python:acceptance --artifacts`; CI retains its existing Product/Tests inputs.
- Windows dev client wheel: 5,645,209 bytes (`cp313-abi3-win_amd64`), built in 277.78 s with 6 local compiler-cache hits and 4 misses. The run included Cargo waiting alongside `check:rust`; this measures the client surface, not the later backend and authoring wrappers.
- Windows moved-client measurements: 176 unit tests in 24.24 s, 295 integration tests in 42.10 s and two native acceptance stories in 552.49 s, using the focused CLI/plugin fixtures. Installed-wheel import and strict stubtest passed for all 39 shipped modules on CPython 3.15.

## Inherited Mechanisms

The gRPC service, Harmos sidecar lifecycle, `local.lock`, reports and Python-client behaviour remain. Client parity replaced `packages/dokime-client`, generated Python wire code and descriptors. Native platform wheels replace the pure `Build / Product / Python (any)` producer.
