# Query Operations Delta

## MODIFIED Requirements

### Requirement: Canonical Read Operations

Dokime SHALL expose one canonical resource read path per data kind, replacing
overlapping operational, query and checkpoint families. Each SHALL have one
declaration per required face, generated request/response wrappers and
runtime-owned execution.
Reads SHALL be read-only and idempotent at an explicit compatible returned cut;
latest reads SHALL select already durable data without a flush barrier.

#### Scenario: Initial Live Query

- **WHEN** a query resolves a durable cut during a running execution
- **THEN** it does not flush admitted unsaved evidence or change execution lifecycle
- **AND** each product service method delegates through one operation file to a manager or storage, while generic library services are composed directly without redeclaration
- **AND** latest-durable/no-wait reads follow the locked payload-size-independent 250 ms p95 durable-lag bound in the operation-contracts delta

#### Scenario: Contract Discovery

- **WHEN** an authorized caller discovers the operation catalog
- **THEN** each operation has a face/package-derived namespaced ID, its own request schema, response wrapper and declared policy
- **AND** native, gRPC, generated clients and generic CLI calls use the declaration for their permitted face and the locked nested-resource naming grammar

#### Scenario: Plugin and Client Read Produced Records

- **WHEN** a plugin reads produced records through its context and a client reads them through the client API
- **THEN** `dokime.context.v1.evidence.Records/List` and `dokime.v1.evidence.Records/List` have separate request/response declarations and authorization
- **AND** both preserve the same retained-cut and bounded read guarantees without exposing the context face to the client

### Requirement: Cheap CLI Parity

The existing `dokime operation call` command SHALL support canonical resource
read request/response contracts, including `at`, `resolved_at` and page fields,
without a new parser or bespoke query command tree. JSON replies SHALL retain
schema, cut and completeness metadata. Documentation SHALL show a first request
and continuation using the returned token; dedicated flags and dataframe exports
remain outside scope.

#### Scenario: CLI Continuation

- **WHEN** a caller invokes the canonical Evidence.Records.List operation with `dokime operation call`, a request file and an output file
- **THEN** the saved response contains the same typed rows, `resolved_at` and continuation metadata as the language client page interface
- **AND** the next request preserves the returned cut and token rather than resolving latest again
