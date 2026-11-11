# Backend Connections

## ADDED Requirements

### Requirement: Pairing-File Backend List

The desktop and CLI SHALL identify saved remote connections by pairing files. Listing known remotes SHALL list the pairing-file folder, without a profile registry, CLI default profile or credential-store connection secret. Expected backend identity and the pinned server certificate SHALL be checked before use.

#### Scenario: Select a Remote

- **WHEN** a user selects a known pairing file
- **THEN** its ordered endpoints, backend key, certificate and expected identity determine the connection

#### Scenario: Unavailable Remote

- **WHEN** a selected remote cannot be reached
- **THEN** the selection remains visible with connection controls rather than silently starting Local

#### Scenario: Remove the Selected Remote

- **WHEN** the user confirms removal of the selected pairing-file entry
- **THEN** that connection disconnects and no profile or keyring registry is mutated
