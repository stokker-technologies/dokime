## MODIFIED Requirements

### Requirement: Python-Started Backend

While the Python backend API exists, it SHALL start explicitly with a port and configuration and stop in order through public close(), without requiring a context manager. This flat lifecycle SHALL NOT reverse the Python backend removal owned by grpc-plugins.

#### Scenario: Normal Exit

- **WHEN** the caller closes a Python-started backend with no cycle running
- **THEN** listeners and plugins stop and the data root is released

#### Scenario: Port Unavailable

- **WHEN** the requested port is in use
- **THEN** explicit start raises a typed error naming the address and no backend remains
