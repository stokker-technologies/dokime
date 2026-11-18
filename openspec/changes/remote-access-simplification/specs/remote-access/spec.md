## ADDED Requirements

### Requirement: Connected Clients Are Trusted

A connected client SHALL be able to call every operation the backend serves to clients, without grants, scopes or per-operation permissions.

#### Scenario: Second Machine

- **WHEN** a client on another machine connects to a backend
- **THEN** it can start and control executions, watch live plots and read results like a local client

### Requirement: Environment Configuration

The SDK SHALL read `DOKIME_REMOTE` (an `http://host:port` origin or a `.pair` file)
and `DOKIME_PLUGIN_ADDRESS` (default `127.0.0.1:0`). It SHALL load `.env` from the
working directory itself. Configuration precedence SHALL be keyword argument,
process environment, `.env`, default. `.env` SHALL contain addresses only;
secrets SHALL remain in Secret Storage.

#### Scenario: Environment

- **WHEN** the environment names a pairing file and a client connects without arguments
- **THEN** it connects to the backend that file describes

#### Scenario: Nothing Configured

- **WHEN** no keyword argument, process environment, `.env` or default names a backend
- **THEN** the client fails with a message naming the variable to set

#### Scenario: Keyword Argument Overrides Configuration

- **WHEN** the keyword argument, process environment and `.env` provide different connection targets
- **THEN** the SDK uses the keyword argument

#### Scenario: Process Environment Overrides the File

- **WHEN** no keyword argument is supplied and the process environment and `.env` provide different values
- **THEN** the SDK uses the process environment

#### Scenario: Working-Directory File and Defaults

- **WHEN** no keyword argument or process environment supplies a value
- **THEN** the SDK loads that value from the working-directory `.env` or uses its default when the file supplies none

### Requirement: Ordered Pairing Endpoints

A `.pair` file SHALL contain 1 to 32 endpoints. Clients SHALL try them sequentially
in file order with the existing connection/read policy, accepting the first
endpoint that connects and proves the paired identity. Failed channels SHALL
close before the next attempt; identity mismatch SHALL continue. Total failure
SHALL name every endpoint and its failure, with guidance to check the host
listener or export a new pairing.

#### Scenario: First Endpoint Refused

- **WHEN** the first endpoint refuses a connection and the second proves the paired host identity
- **THEN** the client connects to the second endpoint

#### Scenario: All Endpoints Refused

- **WHEN** every endpoint refuses a connection
- **THEN** one actionable error names every endpoint and its failure in file order

#### Scenario: First Endpoint Has a Different Identity

- **WHEN** the first endpoint answers with a different identity and the second proves the paired host identity
- **THEN** the client closes the first channel and connects to the second endpoint

### Requirement: Python Connection Timeout

Python `connect(target=None, *, connect_timeout=30.0, ...)` SHALL accept a positive
finite timeout in seconds. Each endpoint attempt SHALL share that budget across
channel readiness and host identity verification, including a plain URL target.

#### Scenario: Readiness and Identity Share One Budget

- **WHEN** channel readiness consumes part of the configured endpoint budget
- **THEN** identity verification receives only the remaining budget

### Requirement: Command-Line Connection Timeout

The command line SHALL accept a global `--connect-timeout <seconds>` option in
positive whole seconds that overrides `connect_timeout_s` for that invocation;
the stored setting and its 30-second default SHALL remain unchanged.

#### Scenario: Per-Invocation Override

- **WHEN** the caller supplies `--connect-timeout`
- **THEN** that invocation uses the supplied seconds without changing the stored timeout or its default

### Requirement: Complete Pairing Export

One-click pairing export SHALL write every reachable address of the host machine
into the file's endpoint list, using the enabled listener's actual port and pinned
TLS identity. The host SHALL supply the address list to exporters; a listener bind
address alone does not enumerate the machine's reachable addresses.

#### Scenario: Host Has Several Addresses

- **WHEN** an operator exports access to a host with several reachable addresses
- **THEN** the exported pairing contains every reachable address for clients to try in order

### Requirement: Clients Survive Connection Drops

A client SHALL stay usable across a transient connection drop without surfacing an error to its caller.

#### Scenario: Drop During a Watch

- **WHEN** the connection drops and returns while a client watches an execution
- **THEN** the watch continues from where it stopped, with no gap and no duplicate

#### Scenario: Drop During a Request

- **WHEN** the connection drops while a client's request that changes state is in flight
- **THEN** the client repeats it after reconnecting and the backend applies it once

#### Scenario: Backend Gone

- **WHEN** the backend stays unreachable beyond the configured reconnect time
- **THEN** the client reports a failure naming the backend and how long it waited
