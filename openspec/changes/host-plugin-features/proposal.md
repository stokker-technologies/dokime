# Host Plugin Features

## Why

The host sidecar plugin reads Wi-Fi through one combined collection: the link stream borrows congestion from the scan, a scan runs whenever any Wi-Fi view is open, and every probe used to start a console process. Comms-blackout tests also need the host to read its adapters precisely and to switch radios and adapters during an execution, with every change undone afterwards. The owner locked the complete host feature set on 2026-10-29 for the consolidation branch.

## What Changes

Locked by the owner on 2026-10-29.

- **Process launch**: every child process starts through `dokime_domain::process::command` (no Windows console window); Clippy bans `std`/`tokio` `Command::new`. Host probes use native APIs instead of `powercfg` and `ping`, and read the git identity once per collector.
- **Wi-Fi streams (BREAKING, clean break, schemas stay v1)**: `host.wifi` and `host.wifi.channel_occupancy` are replaced by `host.wifi.link` (current connection), `host.wifi.scan` (nearby networks, one row per BSS) and `host.wifi.latency` (RTT and loss to a configured target). Each stream is its own observer with its own lease.
- **Network connectivity**: `host.network` gains `internet` from the OS connectivity state, without own traffic.
- **Spin-down**: a host observer runs only while a lease holds its stream; releasing the last lease cancels its observe invocation.
- **Adapters (read)**: query operation `host.network.adapters`; `host.network` stays compact and publishes change events (link, address, default route) from OS notifications; one adapters snapshot is recorded in execution evidence at run start.
- **Radios and adapters (write)**: `host.network.radio.set` and `host.network.adapter.set` as invertible Harmos transactions, inverted at run end and after a crash, recorded in evidence, guarded against cutting the active remote connection.
- **Remaining host features**: Bluetooth on/off through the radio operation plus device presence; power plan read and invertible set; CPU load in GFLOP/s from a calibrated measure.

## Capabilities

### New Capabilities

- `host-telemetry`: host streams, host queries and host controls of the host sidecar plugin.

## Impact

`dokime-domain` (`process` and generic plot validation), `plugins/sidecars/host` (host stream rows, contracts and cadences, host document defaults, Wi-Fi occupancy binding, collectors, platform code, observers, operations, settings), `dokime-runtime` plot context and test fixtures, the GUI Wi-Fi plot context, the OBC suites and soak template, the product guide's host telemetry section and every crate that starts a child process. Linux paths are proven by CI only.
