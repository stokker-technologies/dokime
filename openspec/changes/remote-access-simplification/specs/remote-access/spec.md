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
