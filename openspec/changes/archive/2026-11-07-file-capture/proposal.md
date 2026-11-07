# File Capture

## Why

`WatchFile` asks a step author for 15 fields, three nested types to name one path and
a hand-called `cancel()` to finish normally. Most fields apply to only some requests,
and the invalid combinations are refused at runtime.

Cancellation is also the normal way to finish, which fails correctly captured work.
`cancel()` on an armed watch can return `OperationRefused(Cancelled)` when sealing
outlasts the operation's `cancellation_timeout`, even though every artifact completes.
`many_file_stop` reproduced this in 4 of 50 runs alone and 5 of 50 under load. The
harness road has the same window at `crates/dokime-runtime/src/execution/instances.rs`.

## Use Cases

The owner named the behavior this change must serve:

1. Pick up the newest file in a folder at start.
2. Stream a live log.
3. Mirror a zip file when it changes during the execution, every 10 s.

## What Changes

- Replace `WatchFile`, `AwaitFileWatchArmed` and the arming ticket with three ordinary
  operations, each called as `ctx.operation(request).await?`: `CopyFiles`,
  `StartCapture` and `StopCapture`.
- Replace `recursive`, `rules`, `selection` and the file-or-directory inference with one
  `FileSource`: `File`, `Tree` or `Newest`, each local or FTP.
- Replace `policy` with `CaptureMode`: `Stream` carries the line rules and `Mirror`
  carries its interval. One-shot copying leaves the mode and becomes `CopyFiles`.
- Make the Mirror interval a real cadence. Today a local Mirror rescans on every native
  notification, so its interval is only a fallback.
- Let the execution own a capture, so it can span steps and seals when the execution ends.
- Record capture gaps as evidence, whether or not the caller reads the outcome.
- Gate the FTP implementation behind an `ftp` feature that shipped products enable.
- **BREAKING** Remove the `FileWatch` gRPC stream, with its frames and generated clients.
- **BREAKING** Move limits and the Stream poll interval from the request to configuration.

## Locked Decisions

- `StopCapture` is an ordinary request, never cancellation. `cancel` stays a true abort.
- `SnapshotAtStop`, `Selection::New` and `wait_for_source_seconds` are removed: they have
  no use case. `Mirror` and newest-file selection stay because the use cases need them.
- FTP stays behind a feature. The wire shape stays unconditional, so a plugin's contract
  never differs from its host's; a build without the feature refuses an FTP source.
- Clean break: no aliases, no compatibility layer and no migration of stored data.

## Inherited Mechanisms

| Mechanism | Outcome |
|---|---|
| Baseline, append and rotation capture, `.segmentN` names | Remain |
| Include and exclude rules, last match wins | Remain |
| Artifact path rules, live Stream artifact reads | Remain |
| Line matchers and markers | Remain, on `Stream` only |
| Last-whole-copy guarantee on a failed Mirror refetch | Remains |
| Runtime-owned sealing and the execution capture barrier | Remain |
| Host and harness capture engines | Remain separate |
| `WatchFile`, `.start()`, `armed()`, `cancel()` for capture | Replaced by `StartCapture` and `StopCapture` |
| `SnapshotAtStart` | Replaced by `CopyFiles` |
| Step-end sweep of watches | Replaced by execution ownership |
| `FileWatch`, `AwaitFileWatchArmed`, `WatchTicket`, `impl StepInstance<WatchFile>` | Removed |
| `SnapshotAtStop`, `Selection::New`, source waiting, `Unspecified` variants | Removed |

## Impact

- **Public surface:** step authors migrate to the new operations. External plugins
  migrate in their own repositories, and the generated Python and GUI clients lose
  `FileWatch`.
- **Crates:** `dokime-domain`, `dokime-operations`, `dokime-runtime`, `dokime-macros`,
  `dokime-server`, `dokime`, `dokime-cli`, `dokime-gui`, `packages/dokime-client` and
  the OBC plugin.
- **Dependencies:** plugin sidecars stop linking `suppaftp`, because capture is served by
  the host and `ftp` no longer comes with `native`.
- **Unchanged:** `RunProcess` keeps `.start()` and `.cancel()`. Storage schemas do not
  change.
