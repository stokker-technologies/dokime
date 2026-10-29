# Runtime And CI

## ADDED Requirements

### Requirement: Shared Server Lifecycle

The CLI SHALL expose `dokime server` using the existing server library, and the GUI SHALL embed that library. The standalone server executable and its packaging SHALL be removed. Process-wide setup SHALL belong to application composition.

#### Scenario: Embedded Restart

- **WHEN** a local GUI session stops or restarts
- **THEN** owned runtime, sidecars and listener resources drain and release within existing bounds without losing data
- **AND** remote attachment and authentication behavior remain unchanged

#### Scenario: CLI Server

- **WHEN** a user or development tool starts `dokime server`
- **THEN** existing configuration, network, version and JSON control behavior remains available without a companion server executable

### Requirement: Deterministic Test Credentials

Tests SHALL inject an isolated in-memory credential provider at runtime composition. Production SHALL retain OS-backed secure storage and fail clearly when unavailable, with no insecure fallback.

#### Scenario: Linux Without Desktop Services

- **WHEN** settings, installed-plugin and diagnostics integration stories run without a desktop keyring service
- **THEN** their unchanged behavioral assertions pass with the explicitly supplied test provider
- **AND** production secure-storage and existing-data guarantees are unchanged

### Requirement: Default Profiles And Nightly Consumer Proof

Project compiler overrides SHALL be removed. Tests SHALL use default test compilation with matching Clippy and consume the existing archive for contract execution. The full generated-starter consumer proof SHALL run only in an explicitly declared nightly test job.

#### Scenario: Ordinary Candidate

- **WHEN** PR or dev validation runs
- **THEN** parallel Product and Tests use their default profiles and ordinary scaffold tests remain
- **AND** no full generated-starter build or redundant GUI compilation occurs during contract execution

#### Scenario: Nightly Starter

- **WHEN** the nightly starter job runs
- **THEN** it consumes the declared Product, generates and compiles a real consumer and fails on a broken starter
- **AND** its duration is visible in pipeline accounting

### Requirement: Owned Report Browser Cleanup

Report testing SHALL open reports in a test-owned browser instance/profile and close each owned tab on success, failure and timeout before deleting temporary report files. Production opener behavior SHALL remain unchanged. Unrelated user tabs and browser processes SHALL never be closed.

#### Scenario: Failed Report Journey

- **WHEN** a journey opens a report and then fails or times out
- **THEN** its owned browser tabs close before the report fixture is removed
- **AND** the actual report-open and content assertions remain required

#### Scenario: Locked Design Decisions Apply

- **WHEN** this requirement is implemented or reviewed
- **THEN** the following locked design decisions hold unchanged:

Native report journeys supply a test-scoped opener resource targeting an owned browser profile and debugging endpoint. The gate records exact report tab identities and closes those tabs in its final cleanup before deleting fixtures, including failure and timeout paths. Production keeps the system opener. No user browser enumeration, tab closure or process termination is permitted.

### Requirement: Keyring Secrets And Base64 Files

Host secrets SHALL use the OS keyring, existing catalog and explicit credential-provider composition. Saved files SHALL contain standard Base64 of v1 JSON metadata and values, without encryption or passwords. Legacy adoption SHALL be removed without reading or deleting old stores. Unrelated authentication and cryptography SHALL remain unchanged.

#### Scenario: Saved File Roundtrip

- **WHEN** selected keyring secrets are exported to a .b64 file, read back and imported through runtime operations
- **THEN** one focused in-process story verifies the saved Base64 document and restored values using isolated credential stores
- **AND** no age codec, password-dependent transfer API or legacy-source adapter remains
- **AND** the runtime bounds decoded documents to 512 KiB and encoded files to four times the rounded-up third of that limit
- **AND** imports retain generation checks and explicit replace/skip choices

#### Scenario: Native Settings Secret Management

- **WHEN** the native Settings story adds and removes two synthetic secrets
- **THEN** the real keyring-backed UI reflects both changes with masked values
- **AND** no separate encrypted-transfer UI journey or superseded crypto/provider test suite remains

#### Scenario: Locked Design Decisions Apply

- **WHEN** this requirement is implemented or reviewed
- **THEN** the following locked design decisions hold unchanged:

Canonical secret values remain in the host OS keyring with its existing catalog and explicit composed credential provider. Remove legacy-source adoption, its runtime/configuration/API/UI surfaces and obsolete Memory/LocalFile backend choices; retain OS-keyring binding semantics and unrelated authentication credentials. Existing legacy stores and files are neither read nor deleted.

The runtime owns standard Base64 encoding of the v1 JSON metadata/value document. Archive.data contains the resulting ASCII file bytes; Export takes names/generation, Preview takes data, and Import takes data/choices/generation. Files use .b64 and no passwords, encryption, ciphertext fields or age dependency. Bound decoded documents to 512 KiB and encoded files to four times the rounded-up third of that limit; keep catalog generation and explicit replace/skip choices. Generic native IPC Base64 wrapping remains transport-only.

One in-process saved-file roundtrip proves this codec with isolated keyring stores. The existing native Settings story adds and removes two secrets. Retire the separate shared-secrets journey, crypto/provider test suite and encrypted fixtures; preserve independent plugin/binding/authentication tests. No watchdog, compiler-profile or native keyring-policy change accompanies this replacement.
