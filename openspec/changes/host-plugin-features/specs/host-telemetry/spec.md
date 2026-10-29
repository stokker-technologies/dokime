# Host Telemetry

## ADDED Requirements

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

- **WHEN** the echo to the configured target receives no reply within half the latency collection bound
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

### Requirement: Invertible Radio And Adapter Controls

`host.network.radio.set` and `host.network.adapter.set` SHALL be invertible transactions whose inverse restores the prior state; every change made during an execution SHALL be recorded with timestamps and inverted at run end, and pending inverses SHALL be applied first when the backend starts after a crash.

#### Scenario: Blackout Window

- **WHEN** a step switches the Wi-Fi radio off and the execution ends by completion, cancellation or failure
- **THEN** the radio is restored and the report shows the off and restore timestamps

#### Scenario: Crash Recovery

- **WHEN** the backend restarts after a crash with a recorded, uninverted change
- **THEN** the inverse is applied before anything else starts

#### Scenario: Adapter Without Administrator

- **WHEN** `host.network.adapter.set` runs without administrator rights on Windows
- **THEN** it refuses with a "requires administrator" error and no elevation prompt appears

#### Scenario: Active Connection Guard

- **WHEN** a step would switch off the radio or adapter carrying the active remote client or backend connection without explicitly allowing it
- **THEN** the change is refused

### Requirement: Power Plan And CPU Capacity

The host plugin SHALL read the active power plan, set it as an invertible transaction, and report CPU load as an absolute GFLOP/s measure derived from a documented calibration.

#### Scenario: Power Plan Restored

- **WHEN** a step sets a power plan and the execution ends
- **THEN** the previous plan is active again

### Requirement: Host Plugin Without Runtime Special Case

The runtime SHALL treat the host plugin as an ordinary installed plugin: no compiled-in installation, no host owner kind and no brand-derived name.

#### Scenario: No Host Plugin Installed

- **WHEN** the desktop starts without the host plugin installed
- **THEN** the runtime boots, lists no host plugin and offers no host telemetry

#### Scenario: Host Plugin Installed From A Release

- **WHEN** the host plugin bundle from a Dokime release is installed
- **THEN** it appears under its manifest name and its streams are owned like any other plugin's

### Requirement: Organization Pack Bundled Plugins

Organization installers SHALL bundle the `.dokime` files of the pack's `plugins/` folder, verified at packaging time, and install them on first start under their own names without downgrading newer installed versions.

#### Scenario: Incompatible Bundled Plugin

- **WHEN** a bundle in `plugins/` targets another platform or an incompatible Dokime version
- **THEN** installer packaging fails and names the bundle

#### Scenario: Installer Upgrade With Newer Installed Plugin

- **WHEN** an installer upgrade bundles an older version of a plugin than the installed one
- **THEN** the installed version is kept

