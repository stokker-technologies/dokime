# File Capture Design

Everything here is proposed behavior unless labelled as landed.

## Caller Code

### Newest File at Start

```rust
// Copy Newest Log
let source = FileSource::newest(&log_folder).include("*.log");
let request = CopyFiles::new(source, "logs");
let copied = ctx.operation(request).await?;
```

A step that wants to follow that file instead passes the same source to
`StartCapture::stream`.

### Live Log Stream

```rust
// Start Log Stream
let source = FileSource::file(&self.log);
let request = StartCapture::stream(source, "device/device.log").matcher(LineMatcher::publishing::<DeviceLog>(PATTERN));
let capture = ctx.operation(request).await?;

// Produce Readings
self.device.sample(0.0);

// Stop Log Stream
let request = StopCapture { capture_id: capture.capture_id };
let captured = ctx.operation(request).await?;
```

### Zip Mirror Every 10 s

```rust
// Start Archive Mirror
let source = FileSource::file(&self.archive);
let request = StartCapture::mirror(source, "device/results.zip").interval_s(ctx.parameters().archive_interval_s);
let capture = ctx.operation(request).await?;
```

The value comes from the plugin's settings or parameters, never a literal. This capture
may run across later steps. The execution seals it if no step stops it.

### One File as It Stands

No new operation is needed. This already exists, runs in the plugin's process and has no
FTP or capture limits:

```rust
ctx.artifacts().import(&self.log, "device/boot.log")?;
```

## Public Shapes

```rust
// dokime.files.copy_files
pub struct CopyFiles {
    pub source: FileSource,
    pub output: String,
}

// dokime.files.start_capture
pub struct StartCapture {
    pub source: FileSource,
    pub output: String,
    pub mode: CaptureMode,
}

// dokime.files.stop_capture
pub struct StopCapture {
    pub capture_id: u64,
}

pub enum FileSource {
    File(FileLocation),
    Tree(FileSelection),
    Newest(FileSelection),
}

pub struct FileSelection {
    pub root: FileLocation,
    pub rules: Vec<FileRule>,
}

pub enum FileLocation {
    Local { path: String },
    Ftp { url: String, secret_id: Option<String> },
}

pub enum CaptureMode {
    Stream { matchers: Vec<LineMatcher>, markers: Vec<LineMarker> },
    Mirror { interval_s: Option<u32> },
}

pub struct CaptureStarted {
    pub capture_id: u64,
}

pub struct CapturedFiles {
    pub artifacts: Vec<CapturedArtifact>,
    pub gaps: Vec<CaptureGap>,
}
```

`CopyFiles` and `StopCapture` both answer `CapturedFiles`. Authors build requests only
through constructors, so a wrapper that `harmos::message` needs for a data-carrying enum
stays out of their code.

`output` is the full artifact name for `File`. For `Tree` and `Newest` it is the prefix
under which relative paths are preserved, and empty means the artifact root.

## Sources

- **`File`** names one file. A missing file fails at once.
- **`Tree`** names a directory and its ordered rules. Empty rules select every file. An
  accessible empty directory arms at once and captures later files from byte zero, which
  also covers a log the producer creates after the start. A missing root fails at once.
- **`Newest`** takes the same selection and picks the newest match at the start by
  modification time, breaking ties by path. It follows that file's rotation and never
  switches to a later arrival.

**Behavior change:** `Newest` with no match now fails. Landed `Latest` arms on nothing and
captures nothing, and evidence should not go missing silently.

The rules replace `recursive`. Discovery stays at the top level when no rule contains `/`
or `**`, and descends otherwise, so `*.log` never walks a large tree.

## Lifecycle

```text
StartCapture ── baseline taken ──> CaptureStarted { capture_id }
                                        │
         StopCapture ── final pass, seal ──> CapturedFiles
         execution end ── final pass, seal ──> artifacts and gap records
```

- **`StartCapture`** returns once the baseline is taken, replacing `armed()`. It declares
  no replay, because a replay would start a second capture.
- **`StopCapture`** takes the final pass, seals and answers. It is idempotent: a repeat
  returns the same outcome, and it declares that replay safety.
- **Stop deadline:** `StopCapture` uses `FOLLOW`. Capture limits bound a seal, not a wall
  clock, matching the rule for transfers. Sealing is runtime-owned work, so cancelling
  the call or passing its deadline abandons only the reply. A later `StopCapture` still
  returns the outcome, and a run stop still waits on the execution's capture barrier.
- **Ownership:** a capture belongs to the calling step's execution. Without an execution
  in the context, the call is refused, as is a stop from another execution. A
  capture left running seals at execution end without a warning, because "during the
  execution" is a named use case.
- **Registry:** capture state lives in the execution's registry until it seals, and its
  capacity comes from configuration.

This removes the cancel race on the normal path: no finishing call passes through
`OperationPolicy.cancellation_timeout`.

## Mirror Cadence

Landed `wait_for_poll` wakes on a native change or the interval, whichever comes first,
so a local Mirror refetches a growing zip after each notification.

`Mirror` will check for a size or modification-time change once per interval and ignore
notifications. The stop or execution-end pass always runs. `interval_s` overrides one
call, and its default and bounds come from configuration. `Stream` keeps
notifications plus the configured poll fallback, because it wants low latency.

Known limit, unchanged: a Mirror refetch copies whatever is on disk, so an interim copy
can catch a half-written zip. The final pass supersedes it, and a mirrored artifact
becomes visible when it seals, not while it runs.

## Gaps as Evidence

Landed gaps reach only the watch outcome, which the sole non-test caller discards. Both
engines will publish a warning record of a new built-in type,
`dokime.files.capture_gap`, in the capture's execution scope, carrying path, offset and
reason. `CapturedFiles.gaps` still reports them to a caller that stops the capture.

## FTP Feature

```toml
# crates/dokime-runtime/Cargo.toml
ftp = ["native", "dep:suppaftp"]
testing = ["native", "ftp"]

# crates/dokime/Cargo.toml
ftp = ["dokime-runtime/ftp"]
```

`dokime-server` enables `dokime/ftp`, and the desktop and CLI inherit it through their
`dokime-server` dependency. The feature gates `domains/files/ftp.rs`, the FTP source
variant and the FTP dependencies. `ResumeUnsupported` stays on the wire with FTP's existing
behavior. Without the feature, an FTP location is refused with the existing `UNSUPPORTED`
code. The harness keeps refusing FTP.

## Configuration

Limits and cadence move from literals and request fields to typed configuration, each
with one default, carrying over today's values unless noted:

| Setting | Default | Replaces |
|---|---|---|
| `maximum_files` | 64 | Request override and literal |
| `maximum_bytes` | 256 MiB | Request override and literal |
| `maximum_captures` | 256 | Readiness capacity literal |
| `mirror_interval` | 10 s, new | `poll_interval_ms` for Mirror |
| `storage.filesystem.poll_interval` | Unchanged | `poll_interval_ms` for Stream |

Removing `AwaitFileWatchArmed` also removes its literal 30 s and 86,400 s waits. The
remaining literal limits in `watch_settings` and the harness move into the same section
during implementation; their bounds are validated once, at configuration load.

A file over `maximum_bytes` still fails explicitly and keeps what was captured. A bench
mirroring a larger zip raises the configured value.

## Harness Road

The harness keeps its own local engine. `served` routes all three operations for an
unhosted call. Its registry lives on the binding and seals at run end, and the `Ready`
parameter leaves `Serve`. Merging the two engines is out of scope: FTP keeps them apart.

## Removal Inventory

- **Operations:** `WatchFile`, `FileWatch`, `AwaitFileWatchArmed`.
- **Wire:** `FileWatchFrame` and its payloads, `FileWatchPhase`, `FileWatchPolicy`,
  `FileWatchSelection`, `FileWatchArmed`, `WatchFileOutcome`, and the `Unspecified`
  variants of `FileRuleAction`, `LineFieldKind` and the gap reason.
- **Runtime:** `WatchTicket`, `Readiness`, `LocalRun.readiness`, `impl StepInstance<WatchFile>`,
  the `O::ID == WatchFile::ID` branches in `execution/calls.rs` and `execution/instances.rs`,
  and the `FileWatch` request path in `domains/files`.
- **Server:** `crates/dokime-server/src/grpc/files.rs`, the `Files` service, the
  `LOCAL_ONLY` entry and `tests/integration/execution_artifacts/file_watch.rs`.
- **Clients:** `FileWatch` in the generated GUI client and `packages/dokime-client`.

## Rejected

- **Constructors over `WatchFile`:** hide the fields but keep the wire shape, the ticket
  and the cancel race.
- **A Harmos graceful-stop primitive:** fixes the race but changes Harmos, the wire
  protocol and every SDK.
- **`ctx.capture(source, |ctx| ..)`:** fights the `&mut ctx` borrow and adds a second call
  syntax, against the locked one-invocation decision.
- **A `#[cfg]` on the wire variant:** makes a plugin's contract depend on its build.
