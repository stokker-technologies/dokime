# Host CPU Load

## Why

Suites need a cancellable, measured host CPU workload for repeatable interference tests.
The existing `host.cpu.load` query estimates load but generates no work.

## What Changes

- Add Public `host.cpu.load.hold` with typed percent or generator FLOP/s targets, optional worker count and a cancellation summary.
- Reuse the startup calibration and its exactly counted floating-point kernel.
- Give CPU delta sampling one fixed-cadence owner shared by telemetry and control reads.
- Publish controller evidence on distinct `host.cpu.load.control`, with structured units and bracketed labels.
- Add an OBC start → hold → cancel example and update plugin guidance.

## Impact

Owner design locked on 2026-10-30, including the shared-sampling amendment.
`plugin-host` owns definitions, settings, native workers, sampling and controller evidence.
`plugin-obc` consumes the Public contract. Existing estimated system GFLOP/s remains an approximation.
No system-wide FLOP counter, storage migration, effect inverse, push or PR is included.
Owner resume adopts Harmos 4e2669fa typed cancellation responses and portable `Option<u32>` threads.
