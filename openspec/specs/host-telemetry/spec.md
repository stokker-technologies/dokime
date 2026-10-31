# host-telemetry Specification

## Purpose
Defines the host telemetry streams, including separate Wi-Fi streams and internet connectivity observation.

## Requirements

### Requirement: Windowless Child Processes

Every Dokime child process SHALL start through `dokime_domain::process::command`, which SHALL start Windows children without a console window, and the workspace lint configuration SHALL refuse any other `Command::new`.

#### Scenario: Desktop Starts A Probe

- **WHEN** the desktop or a plugin starts a console program on Windows
- **THEN** no console window opens

### Requirement: Separate Wi-Fi Streams

The host plugin SHALL publish `host.wifi.link`, `host.wifi.scan` and `host.wifi.latency` as separate streams, each collected by its own observer under its own lease, and SHALL NOT publish `host.wifi` or `host.wifi.channel_occupancy`.

#### Scenario: Link Without Scan

- **WHEN** only `host.wifi.link` is leased
- **THEN** the link observer runs and no active scan or echo is performed

#### Scenario: Scan Rows

- **WHEN** `host.wifi.scan` collects
- **THEN** it publishes one row per observed BSS with SSID, BSSID, signal, channel, band, security and whether the host is associated with it

#### Scenario: Minimum Scan Interval

- **WHEN** a scan is requested sooner than the configured minimum scan interval after the previous active scan
- **THEN** no active scan is started and the cached BSS list is reported as cached

#### Scenario: Latency Loss

- **WHEN** the echo receives no reply within the configured echo wait, whose default is half the latency collection bound
- **THEN** the sample reports full loss instead of failing the collection

### Requirement: Internet Connectivity

`host.network` SHALL report `internet` from the operating system's connectivity state without sending its own traffic.

#### Scenario: Connected Host

- **WHEN** the operating system reports IPv4 or IPv6 internet connectivity
- **THEN** `internet` is true

### Requirement: Lease-Bound Host Observers

A host observer SHALL run only while a lease holds its stream, and releasing the last lease SHALL cancel its observe invocation.

#### Scenario: Idle Host

- **WHEN** no stream of the host plugin is leased
- **THEN** no host observer runs

#### Scenario: Viewer Releases

- **WHEN** a viewer holding the only lease on one host stream releases it
- **THEN** that stream's observe invocation is cancelled and no other host observer ran meanwhile

### Requirement: Host Adapter Inventory

The host plugin SHALL answer `host.network.adapters` with every adapter's identity, kind, status, addresses, gateways, DNS servers, DHCP state and whether it carries the default route, and an execution SHALL record one snapshot at run start.

#### Scenario: Run Start Snapshot

- **WHEN** an execution starts
- **THEN** its evidence holds the adapters snapshot taken at start

### Requirement: Network Change Events

`host.network` SHALL publish link, address and default-route change events from operating-system notifications without polling.

#### Scenario: Default Route Moves

- **WHEN** the default route moves to another adapter
- **THEN** a default-route change event is published

### Requirement: Host Native Probe Settings

Host native probe tunables SHALL be declared through plugin settings in `settings.rs`, with validated bounds and defaults preserving current behavior. Settings SHALL resolve at plugin startup and apply on restart or Retry. Compile-time SDK declarations SHALL retain explicit centralized defaults until the SDK supports runtime configuration; unused editable settings SHALL NOT imply unsupported behavior.

#### Scenario: Configured Bluetooth Cache Refresh

- **WHEN** a configured deep-refresh interval has not elapsed
- **THEN** the collector reuses its cached snapshot without probing again

#### Scenario: Refresh Becomes Due

- **WHEN** the configured deep-refresh interval elapses
- **THEN** the collector probes again and updates its cache

#### Scenario: Invalid Native Probe Setting

- **WHEN** a saved timing is zero or exceeds its declared bound
- **THEN** settings resolution refuses it before collector startup
