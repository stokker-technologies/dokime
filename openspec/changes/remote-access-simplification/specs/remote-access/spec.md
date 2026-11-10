## ADDED Requirements

### Requirement: Connected Clients Are Trusted

A connected client SHALL be able to call every Public operation without grants, scopes or per-operation permissions.

#### Scenario: Second Machine

- **WHEN** a client on another machine connects to a backend
- **THEN** it can start and control executions, watch live plots and read results like a local client

### Requirement: Environment Configuration

A client SHALL take its connection from environment variables or from a pairing file carrying the same values.

#### Scenario: Environment

- **WHEN** the connection variables are set and a client connects without arguments
- **THEN** it connects to the endpoint they name

#### Scenario: Nothing Configured

- **WHEN** no variable and no pairing file names a backend
- **THEN** the client fails with a message naming the variables to set
