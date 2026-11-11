## ADDED Requirements

### Requirement: Shared Canonical Query Computation

Identical canonical plot/card queries at the same readable cut SHALL share computation across watches and clients. Identity SHALL include scope, source generation, schemas, selection, filters, resolution and exactness, excluding display IDs. In-flight work and result retention SHALL be bounded with eviction. Each consumer SHALL own cancellation and backpressure; slow consumers SHALL NOT delay peers.

#### Scenario: Two Clients Share One Plot

- **WHEN** two clients request the same canonical plot at the same readable cut concurrently
- **THEN** one backend query computation serves both with equivalent results

#### Scenario: Different Cut or Selection

- **WHEN** requests differ in readable cut, scope, schema, filter or resolution
- **THEN** they cannot reuse a result from the incompatible request

#### Scenario: Slow or Cancelled Client

- **WHEN** one client stalls or cancels while another still needs the shared query
- **THEN** the other completes independently and bounded retention/eviction prevents the stalled client from retaining unlimited work or results
