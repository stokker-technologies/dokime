## ADDED Requirements

### Requirement: Immutable Build Publication
The serialized publisher SHALL consume successful native and Python packages
from the exact dev CI commit and publish `v<version>-dev.<UTC YYYYMMDDHHMMSS>.g<sha8>`
as a non-latest pre-release before moving `dev-channel`. Both releases SHALL
receive the same staged assets and SHA256SUMS. Notes SHALL identify the commit,
PR title and source CI URL. Existing immutable tags SHALL never be overwritten.

#### Scenario: Complete Or Interrupted Upload
- WHEN all assets and checksums have uploaded and verified
- THEN the draft becomes a visible pre-release and channel replacement may proceed
- AND an upload or verification failure leaves the immutable release drafted and the channel unchanged

#### Scenario: Existing Tag
- WHEN the proposed immutable tag or release already exists
- THEN publication refuses it without replacing its tag or assets

### Requirement: Owned Dev Retention
`release.dev_release_retention` SHALL select a positive retention count, default
20. Only published pre-releases with the exact dev tag shape, workflow marker
and native workflow bot author SHALL be eligible. Ordering SHALL use the UTC
build timestamp in the tag. Stable releases, foreign releases, drafts and
`dev-channel` SHALL remain untouched.

#### Scenario: Retention Boundary
- WHEN eligible pre-releases exceed the configured count after successful publication
- THEN only the older eligible releases and their exact tags are deleted
- AND releases outside the owned dev set are preserved

### Requirement: Stable Isolation And Consumer Pins
Dev tags SHALL not trigger the stable Release workflow or qualify as stable
release evidence. Existing release CI and dependency gates SHALL remain.
Documentation SHALL show mise versions equal to the exact dev tag, matching
the source dependency revision, and explain the default retention window.

#### Scenario: Dev And Stable Tags
- WHEN a dev tag is published
- THEN stable release operations do not start
- AND ordinary stable version tags retain their existing release gates
