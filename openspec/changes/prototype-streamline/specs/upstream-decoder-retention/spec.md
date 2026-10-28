# Upstream Decoder Retention

## ADDED Requirements

### Requirement: Bounded Bundle Installation Decoding

Streamed plugin bundle installation SHALL complete under unchanged transfer limits by consuming the published Harmos transport-only backport.

#### Scenario: Install The Matched Bundle

- **WHEN** the native desktop installs the matched 35,284,480-byte OBC bundle with unchanged 1 MiB message/stream and 4 MiB decoded-retention settings
- **THEN** decoded-item retention does not reject the installation
- **AND** genuine over-limit input is still rejected

#### Scenario: Claim Installation Proof

- **WHEN** installation or initialization proof is claimed
- **THEN** matched native product and test artifacts were rebuilt for the candidate

#### Scenario: Locked Design Decisions Apply

- **WHEN** this requirement is implemented or reviewed
- **THEN** the following locked design decisions hold unchanged:

The native desktop installs the matched 35,284,480-byte OBC bundle through `Client::plugins_install_bundle`, using the backend's advertised chunk size and unchanged 1 MiB message/stream and 4 MiB decoded-retention settings. The backend rejects it with ResourceExhausted: `gRPC decoded items retain byte capacity`. The handler drops its header before receiving chunks and drops each consumed frame. At storage checkpoint `7832a106ca47b7a7a1d742274caa19364a8322e5`, Harmos `grpc/decoding.rs` retains the complete `Prepared::memory()` peak claim in each forwarded StreamItem, while `grpc/input.rs` decodes the next frame before output backpressure can complete.

Harmos now provides the compatible transport-only backport on the pinned v0.1.6 storage API line. Dokime consumes that published Git revision through all four direct dependencies and all eight locked Harmos packages; the lock refresh succeeds without an authoring or Arrow migration. Preserve genuine over-limit rejection, transfer limits and deadlines. Matched native product and test artifacts must be rebuilt before claiming installation or initialization proof. The broader authoring migration remains deferred.
