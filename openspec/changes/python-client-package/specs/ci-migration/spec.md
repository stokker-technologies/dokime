# Python Client Publication

## ADDED Requirements

### Requirement: Publish The Distinct Python Client
The Python distribution SHALL be dokime-client and expose only the dokime_client import package. Installation documentation SHALL use uv. Rust crate, protocol, product and executable names SHALL remain unchanged.

#### Scenario: Install And Import The Client
- WHEN a user installs the built dokime-client wheel through uv
- THEN dokime_client and its generated wire modules import with the existing client behavior
- AND the wheel metadata names dokime-client and contains no old dokime import package

#### Scenario: Reproduce And Publish The Client
- WHEN the producer regenerates bindings, builds distributions and packages the candidate
- THEN the renamed uv workspace and artifact consumers use the same distribution/import contract
- AND the existing PyPI publisher expects stokker-technologies/dokime, managed-release.yml and the release environment

### Requirement: Use The Ordinary CI Job Budget
Ordinary CI jobs SHALL inherit the canonical 120-minute default without unnecessary per-job overrides. Exceptions SHALL have a concrete hang/deadlock justification and thirty-minute increments. Individual test deadlines and required checks SHALL remain unchanged.

#### Scenario: Render Required Jobs
- WHEN the candidate regenerates its native CI graph
- THEN ordinary jobs use the canonical 120-minute budget
- AND test failures remain blocking within their existing story deadlines
