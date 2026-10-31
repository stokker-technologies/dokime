# Test Progress And Fixture Ownership

## Proposed Implementation

The SQLite telemetry flush worker observes the existing saved cursor as well as
its configured deadline and tail changes. Rows already saved no longer retain
a timer task through teardown. Buffered execution publications flush on scheduler
progress instead of waiting for a fixed sleep; explicit step/record/stop barriers
retain their ordering and durability.

Local file capture subscribes through Harmos's existing native directory watcher
before its baseline scan. Native events request a new scan; the configured poll
interval remains the fallback upper bound. FTP retains polling because no local
native signal exists. Cancellation and execution sealing keep their own signals.
Readiness registrations belong to active watches and retire on settlement.
Rejection/cancellation guards inspect their operation result; the existing
channel-settlement guard owns subscribed-readiness failure. A late lookup after
joining a finished watch cannot substitute for that guard or wait out registration.

OBC is a test-only simulated plugin. Its short observer cadences have one owner
in `settings.rs`; model sample intervals remain model inputs. Sample-count waits
observe post-publication progress, so completion cannot precede accepted rows.
Shipping host settings and operation policies retain their defaults.
Host fixture overrides shorten observer intervals only. CPU calibration retains
its default compute window so startup still completes real calibration work.
The startup log producer cancels and seals its watch after writing its readings,
retaining the operation's safety deadline instead of waiting for it to expire.

## Fixture Boundaries

`crates/dokime-runtime/tests/support/settings.rs` owns fixture cadence and small
retention/page boundaries. Runtime, server and desktop Rust fixtures consume it.
File polling retains the fifty-millisecond admission minimum; native events wake
local captures sooner. Other fixture publication cadences use one millisecond.
Eviction guards derive workloads from configured retention; paging guards retain
the first row beyond their configured page. Healthy record traversal across
transaction-only gaps folds into the retained-history story, retiring the
duplicated large-gap story. Artifact frame/backlog boundaries remain intact.
Independent registrations at the 257-output boundary run concurrently so storage
can batch their admissions; every file and the final seal assertions remain.

The matrix cases share one backend within each independently selected story.
Each case retains its own cycle, execution, client and lifecycle assertions;
artifact selection includes execution identity. Export reads use useful bounded
chunks while preserving cancellation, chunk-size admission, offsets and digests.
Matrix prompt and completion waits observe runtime changes; record/sample visibility uses each
evidence frontier's own stream. Existing idle-query observation windows remain:
they detect forbidden periodic requests rather than delay observable progress.

## Desktop Ownership

One native desktop lane exclusively owns the interactive session. Its independent
workflows run concurrently with their existing unique temporary roots, ports,
browser profiles, process ownership and JUnit destinations. The launcher joins
every workflow and retains failures. Native picker actions address their owning
process/window. Observable DOM changes replace fixed rendering sleeps.
Backend fixtures wait for their listener instead of repeatedly launching CLI
health probes. Normal startup verifies selected plugins before binding that
listener. Startup mode, its budget, stdin-owned shutdown, real client connections
and readiness evidence remain intact.

## Measurement And Delivery

Baseline binaries remain fixed while fresh traces are collected. Before every
lane, inspect active nextest and test-lane processes and wait for other runs.
Preserve baseline traces before final runs replace them. Comparisons and measured
upstream lifetime shares belong under `target/`, not maintained progress reports.
Overlapping async spans are not additive CPU or wall time. Harmos remains unchanged.
The owner's explicit proof is workspace check, format, and final integration and
acceptance lanes, followed by a local commit without pushing.
