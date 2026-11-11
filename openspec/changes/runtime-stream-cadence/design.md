# Runtime Stream Cadence

## Approved Contract

```rust
// Required Initial Cadence (Const Expressions Accepted)
#[dokime::observer(interval_ms = DEFAULT_INTERVAL_MS)]
impl Poller {
    // Optional Activation Override
    fn interval(resources: &ExecutionContext) -> Result<Duration, Error> { /* settings */ }
}
```

Harmos resolves the producing observer's cadence once per activation, validates a
positive duration, and uses it for initialization retries and observation. Static
observer inventories contain identity, activation, outputs and demand keys only.
The required declared initial cadence remains on Observer::INTERVAL; the default
method returns it when no override exists. Streams stay cadence-less and no
compatibility shim is retained.

Dokime host Settings owns six bounded millisecond intervals. Preparation registers
the resolved settings snapshot; host observers read it through the runtime
interval method. Settings defaults and each manual generic observer's INTERVAL
share constants owned solely by settings.rs. Restart or Retry applies saved changes. Static telemetry schemas
omit nominal cadence rather than advertise defaults as effective values.

## Ownership and Dependency Order

The cadence implementation and released dependency are integrated on
`feat/grpc-plugins`; the workspace pins Harmos 0.1.14 without a local override.
The gRPC change owns subsequent host-clock and SDK migration; final native
consumer proof remains in this change's task list.
Lease admission recognizes authoritative owner observer output membership, while
preserving existing explicit contract.interval publishers. Leases never choose
or retain a sampling interval; the producer alone schedules collection.

## Upstream Capability

The existing SDK forces Observer::INTERVAL and Declared::ENTRY.interval into static
artifact inventories; interval(S::INTERVAL) cannot consume plugin settings.
The supported API cannot supply cadence at runtime. The approved upstream change
adds the optional resource-based override above while retaining declared defaults. The candidate consumes the published dependency graph. No local override or
path-resolved lockfile is part of the handoff.

## Acceptance

Controlled-clock tests prove different configured cadences produce different sample
counts and reject invalid cadence. Settings tests exercise declaration resolution,
existing defaults and validation. No constant-only or wall-clock assertions.
Proof is format, workspace compilation check, and focused plugin-host/SDK tests
through mise tasks only; no release, installers, full lanes or CI simulation.
README impact is reviewed alongside topic guides; update only affected claims.
