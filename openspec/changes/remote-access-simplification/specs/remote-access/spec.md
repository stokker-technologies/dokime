## ADDED Requirements

### Requirement: Connected Clients Are Trusted

A connected client SHALL be able to call every operation the backend serves to clients, without grants, scopes or per-operation permissions.

#### Scenario: Second Machine

- **WHEN** a client on another machine connects to a backend
- **THEN** it can start and control executions, watch live plots and read results like a local client

### Requirement: Environment Configuration

A client SHALL take its connection from a pairing file, named explicitly or through the environment.

#### Scenario: Environment

- **WHEN** the environment names a pairing file and a client connects without arguments
- **THEN** it connects to the backend that file describes

#### Scenario: Nothing Configured

- **WHEN** no variable and no pairing file names a backend
- **THEN** the client fails with a message naming the variable to set

### Requirement: Ordered Pairing Endpoints

A `.pair` file SHALL contain between 1 and 32 endpoints. Every client reading the
file SHALL try them sequentially in file order, using the first endpoint that
connects and proves the paired host identity. Each attempt SHALL use the existing
client connection/read policy and close a failed channel before the next attempt.
Identity mismatch SHALL fail that endpoint and continue to the next endpoint.
After all endpoints fail, one actionable error SHALL name every endpoint and its
failure, with guidance to check the host listener or export a new pairing.

#### Scenario: First Endpoint Refused

- **WHEN** the first endpoint refuses a connection and the second proves the paired host identity
- **THEN** the client connects to the second endpoint

#### Scenario: All Endpoints Refused

- **WHEN** every endpoint refuses a connection
- **THEN** one actionable error names every endpoint and its failure in file order

#### Scenario: First Endpoint Has a Different Identity

- **WHEN** the first endpoint answers with a different identity and the second proves the paired host identity
- **THEN** the client closes the first channel and connects to the second endpoint

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
