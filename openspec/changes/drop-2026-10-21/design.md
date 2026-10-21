# Feature Drop Design

## Crate Layout

`drop-facade` on local branch `drop/facade` owns item 75: the workspace manifests and lockfile; the `dokime` entry library and `dokime-cli` command; Rust server, desktop, packaging and cargo-tool consumers; OBC and starter plugin sources; fixture dependency resolution and macro SDK resolution; Python build/release tooling, package declarations, README and canonical/provider skill copies. No coupled repository changes, GUI TypeScript, generated clients or test behavior changes are included. Implement the entry library before migrating consumers and tooling; the feature-drop coordinator adopts this complete slice into the integration workspace. The command stays `dokime`, distributed by package `dokime-cli`; domain contracts are available under `dokime::domain`, runtime APIs at `dokime::`, and remote access under the optional `client` feature. Product connection, installation, version and error helpers remain entry-library modules; runtime error identity remains available for plugin authors. Acceptance requires repository check, workspace/all-targets check, both changed crates' unit tests and OBC/starter compilation. Integration ownership and provisional publication now belong to the drop workspace; the full drop remains owned by its coordinator.

The approved shared terminal surface stays in the entry library behind `terminal`, because the server and packaging library already consume it. Connection/installation errors are under `dokime::client_error`, while root `dokime::Error` and `dokime::Result` retain runtime identity independently of selected features. Macro expansion resolves either the entry dependency (including a renamed dependency) or the runtime's internal self path. Release assets use `dokime-cli-<version>-<triple>` while retaining executable name `dokime`.

The integration owner also repairs the Python tutorial's exact candidate-wheel lock in `packages/dokime-client/examples/remote/python/uv.lock`. The packaging examples module owns explicit `--relock` authoring from an already built profile candidate, using uv in a temporary tutorial copy and validating before replacing the authored lock. Normal Build and Package retain exact byte validation. Acceptance requires dev Python Build and Package and comparing a freshly built release-profile wheel; different profile hashes require a design decision rather than weakening validation. README owns the contributor invocation. `prepare` restores the shared editable tooling binding to this integration workspace without editing primary-checkout source.

## Integration And Phase

The sole integration workspace is `dokime.wt/drop-2026-10-21`, branch `feat/drop-2026-10-21`. Only this task branch is published; existing slice branches remain local and append-only. Current phase is implementation. Adopt existing dirty changes without resetting or rebasing published history. The entire current numbered inventory, including the watcher extension, requires source accounting before validation; source completion does not establish acceptance.

## Source Ownership

| Workspace | Owned Source And Responsibility |
| --- | --- |
| drop-2026-10-21 | Native report delivery in GUI src-tauri/app and bindings, native-delivery/native-reports consumers, item 49 sidebar branding, combined spec, generated bindings and integration |
| drop-execution | GUI cycle/execution/data queries, server evidence and execution aggregate handlers, corresponding domain operations and tests; item 45 |
| drop-seal | Runtime reports, report operations and note persistence; server/runtime system benchmark workflows; items 21, 32, 43 |
| drop-shutdown | Native close lifecycle and owned host shutdown, matching tests; item 42 |
| drop-runtime-perf | Cargo dev profile/package profile reporting, runtime execution/artifact/telemetry persistence, codec caching, state transitions and prompt notifications; items 44, 47 |

One writer per workspace. All source adoption belongs to the integration owner. Finished plots/settings/branding slices are already incorporated and must be verified in the final tree. Merge outstanding execution, seal, shutdown and runtime work after relevant gates; reconcile overlapping runtime paths explicitly. Regenerate clients from the combined source, never hand-merge generated outputs. Read project architecture and native desktop contracts before changes.

## Locked Behavior

- Prompts live only on the generating execution, in a separate card below Control. Cycle views link to that execution. Shared components retain separate cycle/execution viewers.
- Reports preserve planned member order and pending entries, parameter labels/units, prompt markers, and full series for viewport-sensitive envelope-preserving downsampling.
- Queued cancel and active stop remain distinct. Execution stop does not stop the cycle. Remove continue-after-failure policy; continuation must not re-admit appended cases.
- Item 45 uses bounded server-side record tails and incremental updates, stream invalidation instead of list polling, unchanged-document suppression, aggregate steps/criteria and virtualized lists. Sealed execution detail is stable.
- Item 46 uses message-budget-bounded sequential transfers, native path/channel report delivery without eager base64/HTML duplication, bounded plot points and per-operation heavy mutation deadlines. Client HTTP/2 flow control is tuned; listener tuning is a separate Harmos follow-up.
- Item 47 batches telemetry/artifact durability at valid boundaries, caches telemetry codecs with correct schema identity, avoids whole-state cloning and replaces prompt polling with notifications. Human-approved simplification: preserve per-attempt preparation because current public requests select one attempt; remove unused multi-attempt preparation groups, partitioning and test scaffolding. Preparation retains output ownership and its saved revision before the next cancellation/checkpoint boundary.
- Item 44 applies opt-level 1 to workspace dev packages while retaining debug information/assertions and reports the packaged profile. Measure cold/incremental build cost for user review; do not silently change release semantics.
- Item 42 targets idle close below one second and running close within two to five seconds of settlement, without report rematerialization; attached shells leave remote hosts alive.
- Item 49 places a logo about one sixth of window height above a horizontally centered Dokime wordmark, with a clear gap before navigation.
- Storage remains development v1; no silent data deletion or reset. Preserve current user settings and data roots.

Report delivery uses cycle/execution ArtifactSource references and returns only the delivered path and report origin. The native client verifies subject identity, contiguous bytes, completion and checksum. Final rename/open is serialized with connection-generation invalidation after staging; saved-file opener failure remains a successful save with an actionable warning. Remove the superseded HTML-over-IPC open route and migrate its walkthrough consumer.

Frozen plot payloads retain all captured points, including series above the live query cap. Their display budget applies only after viewport selection, preserving early-window spikes and endpoints. Records use the existing row-window hook so bounded tail reads also stay bounded in rendered DOM size.

## Acceptance And Dependencies

Run relevant checks and writer-owned tests before commits, and `env -u TMPDIR mise run check` before every integration push. Use signed Conventional Commits. No redirected environment roots or externally owned process termination. Run only one full lane per repository at a time and coordinate with the remote-streamline release owner.

After combined source completion, build the development product and required test prerequisites, run affected Rust/TypeScript lanes and native system walkthrough, and inspect the actual mounted workflows on Linux and required Windows CI. Record missing platform evidence honestly. Explicitly requested benchmarks cover 50 MB plugin install, 100k-row report during a run with two prompt answers, ten-member cycle report, telemetry-heavy seal, note during a run, and first plot frame. Measure close latency, idle invokes, 200 MB export, persistence counters and profile build cost. Native reports own evidence; do not fabricate baselines.

Review risks include telemetry-only plots after runtime updates, empty initial plot frames, runtime branding/style ordering, full report fidelity and owned/attached shutdown. Preserve all system story behaviors. README claims and examples must match delivered behavior; no rustdoc or docs authoring before 1.0.

Final review targets dev only after remote-streamline PR 2 lands and published Harmos pins resolve. Authorized delivery includes required native checks, ready review and squash auto-merge; no protected-branch shortcuts. Finish the OpenSpec change when all tasks are satisfied. Existing parked follow-ups (cycle description edit, cycle cancel operation, cancelled-record type, old docs mention, light runtime brand loader) are excluded unless source proves they prevent an agreed item's acceptance. The reported OBC protobuf-support failure is a validation concern, not a waived required lane.

## Watcher Extension

The watcher writer owns disjoint files in the integration tree: domain files models/operations, runtime domains/files, hosted/harness watcher adapters, server file-watch handlers, watcher tests, authoring guidance and examples. Runtime performance retains execution/artifact/telemetry ownership; the integrator owns generated clients, README consolidation and commits. No independent checkout or publication is created.

Selection is independent of byte capture: All remains default; Latest selects the newest matching baseline file by modification time with stable path tie-breaking and follows that file's rotation without switching to later arrivals; New ignores the baseline and captures files first discovered afterward from byte zero. An empty baseline is explicitly represented when arming.

Accessible All/New directory watches arm with an empty baseline even when source waiting is enabled, allowing the producer to create the first file. Latest with an empty strict baseline retains no selection; opt-in waiting defers its baseline until an initial match. Capture duration starts only after successful setup/arming and is independent of the source wait deadline. Failed initial admission never emits successful readiness. Detected replacement and copy-truncate boundaries retain separate segments; undetectable FTP overwrites carry no lossless rotation promise.

Expose capture-armed readiness after baseline establishment to hosted, harness and streaming callers so devices can start without sleeps. Rotation seals and retains distinct segments with unique artifacts; replacement does not consume a cumulative single-file admission cap. FTP resume degradation must fail clearly when matcher/marker extraction cannot continue, preserving evidence and an actionable gap/error; only explicitly signalled artifact-only fallback remains.

Absent-root/file waiting is opt-in, bounded by deadline and cancellation, while strict failure remains default. Permission and invalid-path failures do not become retries. Acceptance covers ready-before-producer, retained rotated segments, FTP capability failure/gap, source appearance within and after timeout, cancellation and deterministic selection. No real-device compatibility claim is implied.

Hosted WatchFile exposes an explicit typed instance readiness await rather than changing generic submit to block. The transport's response-only instance API may require a narrow readiness operation keyed to the existing instance identity; any registry must clean up on completion, failure and cancellation. Multi-file FileWatch emits Armed directly in its progress stream.

Human-approved FTP limitation: status 550 cannot distinguish missing roots from denied access. Unavailable FTP roots fail immediately with an actionable error. Bounded waits cover absent files under an accessible FTP root and absent local roots/files; missing FTP root waiting is excluded.

## Remaining Source Order

First consolidate the original drop/watchers and native report corrections. Finish frontend consumers, generated clients/guidance, README and examples together. Relevant local gates precede commits/publication; complete project validation and benchmarks wait for the entire expanded combined tree. Development schema breaks remain allowed at v1; do not build database compatibility solely for development data, and never silently delete/reset real stored secrets or packs.
