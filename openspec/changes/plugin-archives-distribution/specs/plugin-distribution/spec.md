# Plugin Distribution

## ADDED Requirements

### Requirement: Verified Plugin Release Archives

Dokime SHALL read only plugins.json format 2 and select plugins by declared name,
compatible Dokime version and target. It SHALL verify archive SHA-256 against
SHA256SUMS and member SHA-256 against the feed, inspect the selected bundle identity,
and read only the requested unique regular member. Multiple plugins SHALL be discoverable.

#### Scenario: Shared Archive

- **WHEN** two compatible plugins share one native archive
- **THEN** either plugin can be selected independently and only its verified member is installed.

#### Scenario: Invalid Release Input

- **WHEN** feed v1, incompatible target, corrupt archive/member, unsafe member, link or duplicate member is supplied
- **THEN** acquisition fails before publication and no fallback asset is fetched.

### Requirement: Versioned Distribution Assembly

Installer packaging SHALL resolve an exact release version or dev-channel, default
an omitted version to the running authoring tool version, and acquire the native
distribution ZIP and selected plugin archives from that release. It SHALL verify
release SHA256SUMS and feed member hashes and cache artifacts per user by digest.
It SHALL retain --artifacts build-tree assembly, organization defaults and native
installer assembly without compilation.

#### Scenario: Release And Cache

- **WHEN** an installer selects a release and named plugins
- **THEN** verified distribution and plugin members are assembled, valid cached bytes are reused and corrupted cache bytes are refused or replaced.

#### Scenario: External Plugins

- **WHEN** external_plugins path tables or repeatable --plugin paths are supplied
- **THEN** local bundles receive the same target and requires_dokime checks before assembly.

#### Scenario: Clean Manifest Break

- **WHEN** release_url or sha256 appears in the organization installer manifest
- **THEN** parsing rejects the retired keys.

### Requirement: Product Archive Outputs

Product packaging SHALL emit dokime-distribution-<version>-<target>.zip and
dokime-sidecars-<version>-<target>.zip on Windows or .tar.gz on Unix from existing
build bytes, using unversioned bundle member names from declarations.

#### Scenario: Product Packaging

- **WHEN** matching native build outputs and shipping plugin declarations exist
- **THEN** archives retain exact input bytes and executable modes without compiling, and test-only plugins are excluded.
