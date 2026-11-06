# Design

Foundation, client and backend decisions describe implemented source. Authoring APIs and callbacks remain proposed under tasks 4.x, consuming the separate unified `ctx.operation` API.

## Flow

```python
import asyncio, dokime
async def main():
    async with dokime.Backend(port=0) as backend:
        backend.export_pairing("lab.dokime-remote", label="Bench 3")
asyncio.run(main())
```

## Layout

`sdk/python/` holds Cargo/maturin manifests (private `_native`, abi3), native `src/{client,backend}/`, public `python/dokime/` with `py.typed` and hand-written stubs, `tests/{unit,integration,acceptance,system}/`, and `examples/`. Rust modules mirror public modules; stubs live at public paths, never private Rust paths.

The backend reuses `dokime_server::host::Session`, the managed remote pairing controller, `local.lock` and the normal installed-plugin store. Embedded templates use host-level `dokime.template.*` operations; task 4.1 extends `runtime/embedded.rs::EmbeddedProvider` instead of adding another path.

## Rust To Python Calls

Tokio owns the runtime; Rust releases the interpreter while waiting. Async callbacks run on the starting loop; synchronous callbacks run on workers. Operation deadlines remain authoritative; cancellation cancels the Python task and ignored cancellation is reported. Exceptions become operation failures with tracebacks. Shutdown failures are logged and cannot start new work.

## Fast Streams

`push` serves low-rate data. `await ctx.stream(Measurement).into(Samples).push_columns({"timestamp": timestamps, "voltage_v": voltages})` accepts NumPy or Arrow columns in one crossing with the interpreter released. Fast Stream Support owns and measures throughput targets separately.

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

## Client Parity Decisions

- Rust `Client` owns the live connection, negotiation, identity checks, request deadlines and cancellation. PyO3 async futures release Python while waiting and propagate asyncio cancellation by dropping the Rust future. Closing the Python connection closes the shared Rust handle without stopping admitted backend work.
- Python retains its language-specific owned async contexts, lazy finite readers, immutable value views and optional dataframe conversion. These compose Rust client operations and retain the existing lifecycle and bounded-materialization contracts; no Python gRPC transport remains.
- The client identity snapshot stays at `dokime.client.Backend`; `dokime.Backend` owns the backend async context.
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

## Backend Decisions

- `Session::start` accepts native embedded providers; existing CLI/GUI callers pass an empty list. Python owns one session through an async mutex, released before Python exception construction. Shutdown closes pairing admission synchronously, then uses the server's retained drain; cancelled startup retains the existing rollback. Endpoint/root are immutable startup snapshots. The extension uses the product executable crates' recursion limit of 256: native code generation exceeded Rust's default while computing an inherited runtime-query future layout; check and Clippy did not evaluate that layout.
- `Backend(port=None, root=None, application_root=None, plugins=())` uses the server resolver and default port; zero requests an available local port. `root` selects execution storage, `application_root` selects distribution settings. Installed plugins always load. `client()` returns an independent local client context.
- Synchronous `export_pairing(path, label=...)` releases Python while awaiting the Rust controller and atomically publishing its pairing file. Enabled remote settings remain authoritative; a disabled listener is enabled with TLS and per-client keys at `127.0.0.1:0`, avoiding a second configured-port collision. No network scope is widened implicitly.
- Startup preserves native cause chains and maps owned roots, address conflicts, format identities and installed-ID conflicts to structured Python errors. Classification explicitly unwraps transparent runtime variants and shared native error payloads rather than parsing messages. Installed IDs remain reserved while disabled; removal-pending entries do not reserve IDs. Refusal happens before installed admission can prune a conflicting bundle. The small runtime plugin-admission edit does not touch invocation dispatch.
- Task 3 uses private `_native.EmbeddedProvider(id, version, description)` handles for real metadata-only catalogs with no templates or callbacks. This supports composition and collision proof without defining decorators or an invocation API. Task 4 owns executable authoring and callback support; this handle's `inner` already owns the native provider abstraction.
- Windows dev backend wheel: 40,543,716 bytes (`cp313-abi3-win_amd64`), final task 50.03 s including Clippy, native compilation 25.03 s, three local cache hits and no misses. The preceding contended build took 462.79 s; the first attempt took 1,387.30 s and exposed the repaired native recursion limit. Focused CLI/host/OBC fixture production took 1,675.45 s including queueing; these are workstation measurements, not isolated build benchmarks.
- Windows backend proof on CPython 3.15: nine installed-wheel cases in 57.89 s cover pairing/identity, ordered stop, port/root refusals and cleanup, three unchanged-root format refusals, installed-plugin loading and ephemeral catalog removal, and enabled/disabled installed-ID clashes. Built-wheel import and strict stubtest passed for all 40 shipped modules; Rust and repository checks passed. Later authoring/system proof remains open.

## Authoring Integration

- Fresh `origin/dev` is merged before this slice, including unified `OperationHost` invocation. Python uses `ctx.operation(request)` with builder overrides for one call; declarations retain default policy ownership. Removed generated Python wire files stay removed; the Rust-reflected client consumes updated contracts.
- GUI availability uses the existing live suite catalog after a successful read, matching the execution's template identity. An absent template marks the terminal control card and removes its new-attempt action, while captured evidence continues through the existing document path. Pending or failed catalog queries make no absence claim, and active execution controls remain independent. This applies equally to ephemeral Python and removed native templates; admission still rechecks availability at the backend.
- Windows GUI proof: all 15 selected cycle unit tests and all eight selected lifecycle integration tests pass, including terminal-state availability, pending-catalog behavior, retained running controls and rendering without a new-attempt button. `check:rust`, `check:repository`, TypeScript formatting/lint and `check:typescript:generated --source` pass. The candidate diff's required-term scan is empty. Overlapping GUI task invocations initially corrupted their shared compiler report; both affected selections passed when rerun sequentially. No full simulation ran. README claims remain accurate because Python authoring is still under development.
- Tasks 4.1–4.4 and 4.6 remain open pending the upstream disposition below. No executable Python plugin authoring API, built-wheel callback proof, authoring stubtest result or batch-publication rows/s measurement is claimed by this partial checkpoint. The plain Rust comparison must use the same admitted publication path and workload as the eventual NumPy and Arrow measurements; it is not replaced with a conversion-only benchmark. Tasks 5.x and specification retirement remain deferred to their authorized slices.
- Upstream capability request at the Harmos pin `371108ecfca78d3fe2432f503dd3c355e140ad20`: `Bindings::register` and `Registration::operation` accept compile-time `Executable` types only. `OperationFactory::validate` rejects definitions differing from the compiled declaration, and its contract/handler constructors are private. Python decorators need runtime-owned schemas, policies and handlers. `Manager::submit_encoded` and `PluginCallback::submit_with` do not accept a retry override. `Manager::stage_plugin_observers` is crate-private; the public `stage_observers` uses the root resource scope instead of the admitted plugin generation. The smallest upstream additions are a generation-scoped encoded binding for a validated owned contract, encoded invocation accepting a per-call retry override, and scoped embedded-observer staging. They must reuse Harmos admission, policy, authorization, cancellation and settlement. Fixed-policy adapters, root-scoped plugin observers or a separate Python operation dispatcher would violate the locked semantics. Upstream implementation authorization is pending; the independent GUI work continues. No Harmos source, dependency pin or local override has been changed.
