## MODIFIED Requirements

### Requirement: Label-First Traceability

Traceability sections and rows SHALL derive their identifiers from labels using document
normalization unless the author explicitly sets an identifier. Constructors SHALL accept
labels first without ID-first aliases. The typed `dokime.v1` representation SHALL retain
those identities; the development wire change SHALL NOT require legacy serialization or migrations.

#### Scenario: Derived Identity

- **WHEN** an author constructs a section or typed row with a label
- **THEN** its identifier matches document normalization and its typed data remains intact

#### Scenario: Stable Edit Identity

- **WHEN** an explicit section or row identifier is reused with another label
- **THEN** replacement preserves ordering and unrelated captured content
