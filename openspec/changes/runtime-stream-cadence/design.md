# Runtime Stream Cadence

## Approved Contract

```rust
fn interval(resources: &ExecutionContext) -> Result<Duration, Self::Error>;
```

Harmos resolves the producing observer's cadence once per activation, validates a
positive duration, and uses it for initialization retries and observation. Static
observer inventories contain identity, activation, outputs and demand keys only.
No compatibility shim or static nominal cadence remains.

Dokime host Settings owns six bounded millisecond intervals. Preparation registers
the resolved settings snapshot; host observers read it through the runtime
interval method. Restart or Retry applies saved changes. Static telemetry schemas
omit nominal cadence rather than advertise defaults as effective values.

## Ownership And Dependency Order

Harmos integration: ../../harmos.wt/cadence, feat/runtime-observer-interval.
Dokime integration: this worktree, feat/host-configurable-settings.
Implementation Lead owns Harmos runtime, macros, consumers, tests and docs.
The orchestrator owns Dokime source, both specifications, final checks and commits.
Research And Review owns read-only migration inventory and final review.
Research And Review additionally owns the disjoint lease runtime migration,
lease/capture tests and finalization benchmark fixture cadence (task 1.7).
Lease admission recognizes authoritative owner observer output membership, while
preserving existing explicit contract.interval publishers. Leases never choose
or retain a sampling interval; the producer alone schedules collection.

## Upstream Capability

The existing SDK forces Observer::INTERVAL and Declared::ENTRY.interval into static
artifact inventories; interval(S::INTERVAL) cannot consume plugin settings.
The supported API cannot supply cadence at runtime. The approved upstream change
replaces that path with the resource-based method above. Use an ignored
.cargo/config.toml patch for all resolved Harmos crates during development.
Never commit the override or its path-based lock resolution. Publication and a
verified published Harmos pin are deferred by the owner's commit-only instruction.

## Acceptance

Controlled-clock tests prove different configured cadences produce different sample
counts and reject invalid cadence. Settings tests exercise declaration resolution,
existing defaults and validation. No constant-only or wall-clock assertions.
Proof is format, workspace compilation check, and focused plugin-host/SDK tests
through mise tasks only; no release, installers, full lanes or CI simulation.
README impact is reviewed alongside topic guides; update only affected claims.
