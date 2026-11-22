## Why

Python authors currently assemble bound calls, wire-shaped settings and template-owned telemetry. Replace these surfaces in one breaking migration with typed declarations, ordinary logging and plugin-owned facilities.

## What Changes

- Invoke operations with `await ctx.operation(OP, request=None, policy=None)` and the same client shape; remove BoundOperation.
- Buffer standard logging records in the current execution/scope, flush before host calls and handler return, and route unscoped logs to plugin diagnostics.
- Replace observer decorators and template streams with plugin-level Observer, WindowObserver and Publisher classes, one instance per host sampling period with optional close.
- Declare Setting, Secret, Resource and Record attributes on Plugin; settings use activation snapshots, secrets refuse missing values and resources open once under a lock and close at deactivation.
- Declare typed Parameter attributes on templates; the host validates constraints at case creation and settings constraints at save.
- Classify unexpected handler exceptions as Permanent, with OperationPolicy.transient selecting Transient.
- Use declaration-owned record expressions and observer-owned explicit plots; keep the existing reference_band/reference_line plot form.
- Migrate Python examples and starter to self-contained authoring files. Other SDK language parity remains explicitly open, outside this implementation.

## Capabilities

### New Capabilities

- `python-authoring`: Typed Python authoring declarations and lifecycle facilities.

## Impact

Own SDK Python source except host/** and its generator, Python examples, starter and tests. Host enforcement may change operations/protocol and generated outputs only where necessary. Do not edit docs, GUI, packages, plugins or owner tooling. No compatibility aliases. Missing pinned-library capabilities block their dependent item rather than creating another mechanism.
