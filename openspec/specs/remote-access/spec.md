# remote-access Specification

## Purpose
Define portable remote access files, persistent server trust, discovered endpoints and verified connection and import behavior across native and programmatic clients.

## Requirements

### Requirement: Exact Remote Access File
The system SHALL export and import bounded `.dokime-remote` JSON v1 with exactly
version, endpoints, backend_identifier, label, ca_certificate_pem and access_key.
It SHALL preserve existing secret validation, TLS/CA consistency and 1 MiB limit.

#### Scenario: Round Trip And Rejection
- **WHEN** a valid remote file is encoded then imported
- **THEN** its ordered endpoints, pinned CA, identity and per-file key are retained
- **AND** old single-endpoint files, extra fields, wildcards and malformed trust are rejected

### Requirement: One Export Action
The native desktop SHALL export from one Remote Access button, generate a
readable label, warn that the file is secret and keep its key outside the WebView.

#### Scenario: Disabled Listener
- **WHEN** Export Remote Access is selected with the listener disabled
- **THEN** one confirmation enables its saved address/port before export
- **AND** canceling confirmation leaves it disabled

#### Scenario: Independent Revocation And Rename
- **WHEN** an exported client is renamed or revoked in Paired Clients
- **THEN** only that client's label or access changes, without rotating other keys

### Requirement: Discovered Endpoints And Persistent Trust
The server SHALL discover ordered reachable names from its specific bind or
hostname and real interfaces, ranking private addresses first and excluding
virtual/VPN adapters. Loopback SHALL be labelled same-PC only.
Every exported endpoint SHALL be covered by a leaf signed by the persistent CA.

#### Scenario: Interface Change
- **WHEN** detected interface addresses change
- **THEN** the leaf is reissued under the same persisted CA with the new SANs
- **AND** existing remote files retain trust

#### Scenario: Ranking
- **WHEN** fake interfaces include private, public, loopback, WSL, Docker and VPN addresses
- **THEN** hostname leads wildcard discovery, real private addresses precede public ones,
  and excluded adapters are not exported

### Requirement: Verified Multi-Endpoint Import And Connect
GUI picker/window-drop, CLI `backend add --remote` and Python `from_pairing`
SHALL use all candidates with a configured short stagger and select only a
candidate that passes pinned-CA TLS and expected backend identity negotiation.
Native import SHALL create a profile and OS keyring entry and connect immediately.

#### Scenario: Identity And Trust Gate The Winner
- **WHEN** a fast endpoint presents a wrong CA or backend identity and another is valid
- **THEN** the valid endpoint wins and losing attempts are canceled

#### Scenario: Remembered Candidate
- **WHEN** a saved profile reconnects after a successful connection
- **THEN** its last successful endpoint is tried first, followed by the original order

#### Scenario: Native Import Transaction
- **WHEN** import succeeds through the picker or window drop
- **THEN** the profile contains all endpoints and the key is saved only in the keyring
- **AND** profile persistence failure rolls back its newly saved secret
