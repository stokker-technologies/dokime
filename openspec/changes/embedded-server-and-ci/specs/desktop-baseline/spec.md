# Desktop Baseline

## ADDED Requirements

### Requirement: Authoritative Shutdown Completion
The desktop SHALL retain ownership of an admitted local shutdown until its drain completes. A timeout or failed close attempt SHALL NOT allow a retry to report success solely because the session handle was consumed.

#### Scenario: Retry While Draining
- **WHEN** a local shutdown times out and the user retries
- **THEN** the retry observes the same authoritative drain and reports completion only after owned resources have stopped

### Requirement: Independent Desktop Launches
Every desktop launch SHALL open its own window. Launch arguments and deep links SHALL remain in the launched instance without forwarding. The backend SHALL exclusively lease its data folder and immediately refuse a competing local start with a typed `BackendRunning` error identifying that folder. The connection screen SHALL remain usable and explain that the operator can connect to another or remote backend, or close the other instance and retry.

#### Scenario: Occupied Local Data Folder
- **WHEN** another desktop attempts to start Local on a folder already leased by a backend
- **THEN** it immediately shows the typed refusal and folder while retaining connection and retry controls
- **AND** attaching to another backend remains possible without acquiring Local's lease

### Requirement: Owned Backend Selection
One desktop-session owner SHALL manage Local and saved remote selection. Local SHALL be the initial choice. Saved remotes SHALL reuse the existing backend catalog, and the desktop SHALL remember its choice separately from the CLI default without persisting ephemeral local endpoints.

#### Scenario: Remembered Remote Is Unavailable
- **WHEN** the desktop reopens with an unavailable remembered remote
- **THEN** it presents reconnect and selection controls without silently starting Local

#### Scenario: Ownership-Appropriate Controls
- **WHEN** the user selects or disconnects a backend
- **THEN** controls distinguish starting or stopping the owned Local backend from connecting or disconnecting a remote, and saved profile identity is retained

#### Scenario: Invalid Selection While Connected
- **WHEN** a requested saved profile cannot be resolved while another backend is active
- **THEN** selection fails explicitly and preserves the active connection and its identity

### Requirement: Clean Compatibility Removal
Obsolete route redirects, selection-state helpers and legacy remembered-parameter string values SHALL be removed with their consumers and fixtures. Unsupported stored values SHALL produce an explicit error without automatic migration, reset or deletion. The current typed parameter format SHALL remain supported.

#### Scenario: Unsupported Saved Parameters
- **WHEN** a saved parameter uses the removed string representation
- **THEN** loading reports the unsupported format and preserves the existing file

#### Scenario: Obsolete Route
- **WHEN** an obsolete bookmark is opened
- **THEN** it receives the normal unmatched-route behavior without a compatibility redirect

### Requirement: Minimal Dependency And Projection Surface
Unused GUI sample projections and proven unused dependencies SHALL be removed while live query, watch and server endpoint behavior remains. Dependency features SHALL match required functionality and a deliberate TLS-provider policy; enabling all defaults SHALL NOT substitute for that audit.

#### Scenario: Dependency Audit
- **WHEN** the final source inventory is checked
- **THEN** direct dependencies have verified consumers, duplicate feature suppression is removed, TLS initialization matches the resolved graph, and compatible updates have targeted proof

### Requirement: Direct Plugin Installation
Dropping or selecting a plugin SHALL perform one complete installation without a consent or review step. The domain, runtime, RPC, CLI, desktop and generated clients SHALL expose the same direct Install operation. Superseded Stage and Consent operations, staged state and consent metadata SHALL be removed without compatibility adapters.

#### Scenario: Valid Plugin Drop
- **WHEN** the user drops a valid nonconflicting plugin bundle
- **THEN** installation completes directly and reports the existing restart requirement without prompting for consent

#### Scenario: Invalid Or Conflicting Bundle
- **WHEN** installation receives invalid bytes, a mismatched digest or a conflicting plugin
- **THEN** installation fails without publishing a partially installed or authorized plugin

#### Scenario: Installation Reply Exceeds Its Budget
- **WHEN** the normalized installation reply exceeds the caller's response budget
- **THEN** installation refuses before publishing the new bundle or changing installed settings

#### Scenario: Cancelled Installation Response
- **WHEN** the caller disconnects after installation work is admitted
- **THEN** admitted mutation ownership remains authoritative until completion, and callers reconcile the outcome without an automatic duplicate installation

### Requirement: Independent Plugin Admission
Verified listing and inspection SHALL use bounded read admission independently from serialized mutation admission. Both SHALL retain ownership until their admitted work completes, including transport cancellation; snapshot and bundle verification semantics remain unchanged.

#### Scenario: Installation During Listing
- **WHEN** a verified listing continues after its caller cancels
- **THEN** Install is not rejected solely because that read owns admission, while a second mutation remains subject to mutation serialization
