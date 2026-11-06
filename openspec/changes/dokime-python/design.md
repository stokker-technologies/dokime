# Design

All shapes below are proposed, except where marked as landed.

## Flow

```python
# plugin/__init__.py
import dokime

@dokime.parameters
class Parameters:
    rows: int = dokime.parameter(label="Rows", default=100_000, min=1, max=200_000)
    member: int = dokime.parameter(label="Member", default=0, expansion="cartesian")

@dokime.stream(id="lab.samples", row=Measurement, window=4096, label="Samples")
class Samples: ...

@dokime.template(id="lab.capture", label="Capture", parameters=Parameters, streams=[Samples])
class Capture:
    @classmethod
    async def initialize(cls, parameters, ctx):
        return cls()

    @dokime.step(label="Capture Measurements")
    async def capture(self, ctx):
        await ctx.stream(Measurement).into(Samples).push(row)

lab = dokime.Plugin(id="lab", version="0.1.0", suites=[...])
```

```python
# main.py
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
crates/dokime-python/
  Cargo.toml  pyproject.toml          # maturin, private _native extension, abi3
  src/{client,authoring,domain,runtime}/   # PySomething wrappers, one mod.rs per folder
  python/dokime/                      # public package, py.typed, hand-written .pyi
  tests/{unit,integration,acceptance,system}/
  examples/
```

The Rust module tree mirrors the public Python modules, so `python/dokime/client/`
and `src/client/` correspond. Stubs live at public paths, never under `_native/`.

## Reuse Of Landed Code

| Need | Landed | Change |
| --- | --- | --- |
| Start a backend | `dokime_server::host::Session::start` | Accept embedded providers |
| Pairing file | `create_pairing` in the gRPC remote controller | Expose it; start the listener |
| Plugin provider | `EmbeddedProvider` in `runtime/embedded.rs` | Add operations, observers, sections |
| Client | `crates/dokime-client` | Wrap instead of Python wire code |
| Exclusive root | `local.lock`, `storage::Error::Owned` | Map to a Python exception |

`EmbeddedProvider` declares no operations, observers or sections today, and its
templates run through host-level `dokime.template.*` operations. Extending it keeps
one provider path for Python rather than adding a second one.

## Rust To Python Calls

Tokio owns the runtime; Rust code releases the interpreter while it waits.

- `async def` callbacks are scheduled on the event loop that started the backend.
- `def` callbacks run on a worker thread, so a blocking step does not stall that loop.
- Operation deadlines come from the existing policies and stay authoritative. Cancelling
  an operation cancels its Python task; a callback that ignores cancellation is reported.
- A Python exception becomes the operation's failure with its traceback retained for the
  failure record. It never crosses as a panic.
- Callback failures after shutdown begins are logged and cannot start new work.

## Fast Streams

Pushing one row per Python call cannot meet the stream requirement. `push` stays for
low-rate data, and a batch call accepts column buffers as one host crossing:

```python
await ctx.stream(Measurement).into(Samples).push_columns(
    {"timestamp": timestamps, "voltage_v": voltages}  # NumPy arrays or Arrow columns
)
```

The buffers are read with the interpreter released. Target rates are owned by Fast
Stream Support and measured separately.

## Macro Mapping

| Rust | Python | Why It Differs |
| --- | --- | --- |
| `#[dokime::parameters]`, `#[parameter(...)]` | `@dokime.parameters`, `dokime.parameter(...)` | |
| `#[dokime::template(...)]` on `impl` | `@dokime.template(...)` on a class | |
| `#[dokime::step]`, `observer`, `criterion`, `prompt`, `record`, `stream`, `section`, `suite`, `operation` | Same names as decorators | |
| `#[dokime::catalog] enum` | Lists | No enum dispatch is needed |
| `#[dokime::message]`, `#[dokime::harmos]` | Dropped or a dataclass | The binding handles serialization |
| `ctx.stream::<T>()` | `ctx.stream(T)` | No turbofish |
| `Result<(), Error>` | Raised exceptions | Python convention |

## Storage Format Identity

Clean-break development changes storage without version bumps, so the identity must
not rely on anyone remembering to bump a constant.

- At build time, derive a digest from the declared stored schemas, starting from the
  schema graphs in `harmos::domain::schema::graph` and the SQLite layout.
- On first write, record that digest and the writer's build commit in the data root.
- On open, a different digest refuses with `StorageFormatMismatch`, naming the root,
  the writer build and the running build. A populated root with no marker also refuses.
- Nothing is rewritten, migrated or deleted.

Whether the graphs cover every stored encoding is verified in task 1.1. The digest
fails the task, not the design, if a stored type turns out to be missing from them.

## Build

```text
Build / Product (each platform) -> Build / Python (each platform) -> Python test lanes
```

- `pyo3` is abi3 for Python 3.13+, so there is one wheel per machine.
- The crate is outside the workspace `default-members`, so ordinary builds skip it.
- Reuse needs the same features, profile and target folder as the product build, because
  Cargo resolves features per invocation. Task 1.3 measures the reuse.
- `build:python` takes the machine build lock like every other compile task.

## Inherited Mechanisms

- **Remains:** gRPC service, Harmos plugin lifecycle for sidecars, `local.lock`, reports,
  and `python-client` behaviour, now served by the new package.
- **Replaced:** `packages/dokime-client`, its generated `_wire` code and descriptors.
- **Removed:** the `Build / Product / Python (any)` pure wheel.
