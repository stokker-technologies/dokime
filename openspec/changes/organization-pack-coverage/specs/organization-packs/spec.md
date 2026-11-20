## ADDED Requirements

### Requirement: One Native Installer Producer

Release and organization installers SHALL use the built CLI's installer packager.
Release packaging SHALL consume a same-run local kit with `--artifacts`, preserve
release asset names, and publish verified updater metadata as `release.json`.

#### Scenario: Package the Build

- **WHEN** the Package job assembles a product release
- **THEN** it invokes the built CLI with the repository default manifest and local kit without downloading or compiling

### Requirement: Kit Identity and Default Plugins

Kits SHALL record executable source revision, channel and default plugin names.
Installers SHALL include those defaults unless excluded by name through
`installer.exclude_plugins`; additional named and path plugin entries remain supported.

#### Scenario: Select Default Plugins

- **WHEN** an installer uses a local kit or a downloaded release
- **THEN** defaults come from the local `plugins/` folder or verified release plugin archive respectively, with manifest exclusions applied

### Requirement: Runtime Plugin Bundle Inputs

Bundles SHALL exclude development tool configuration, environment files, version
control metadata, tests, editors, caches and private inputs. Required script inputs
that would be excluded SHALL cause an actionable refusal.

#### Scenario: Bundle a Source Project

- **WHEN** an author bundles a source project with development configuration and tests
- **THEN** the bundle retains its launch/build inputs and runtime metadata without those development members

### Requirement: Installer Packaging Progress

The library SHALL report completed steps and failures through a caller observer.
The CLI SHALL render progress and durations to stderr, retain output paths on
stdout and disable progress for JSON output, without terminal controls in pipes.

#### Scenario: Packaging Failure

- **WHEN** a packaging step fails
- **THEN** completed steps remain visible and the failing step reports its actionable error

### Requirement: Operator Surfaces Follow the Pack

Every surface an operator of an installation sees SHALL show the resolved organization label and artwork instead of the built-in product name.

#### Scenario: First Paint

- **WHEN** the desktop or a report opens in an installation with a pack
- **THEN** its title and icon show the pack's label and logo from the first frame

#### Scenario: Interface Text and Messages

- **WHEN** the interface shows the version row, plugin compatibility text, a file picker filter or an operator-facing error
- **THEN** the text names the pack's label

#### Scenario: Suggested File Names

- **WHEN** the operator exports diagnostics, a support bundle or secrets
- **THEN** the suggested file name derives from the distribution identifier

#### Scenario: Toolchain Unchanged

- **WHEN** an author runs the command-line tool, generates a project or reads a protocol or file format identifier
- **THEN** those keep the Dokime name whatever pack is applied

### Requirement: One Pack Path

An organization pack SHALL be applied only by packaging it into the installer, where it replaces the bundled default.

#### Scenario: Packaged With a Pack

- **WHEN** an installer is packaged with a pack and installed
- **THEN** the installation shows that pack

#### Scenario: No Import

- **WHEN** an operator opens Settings in any installation
- **THEN** there is no way to import, change or remove a pack, and a pack placed in the application folder is ignored

#### Scenario: Packaged Without a Pack

- **WHEN** an installer is packaged without a pack
- **THEN** the installation shows the built-in appearance

### Requirement: Organization Packs and Trusted Release Sources

Organization Packs SHALL provide appearance and organizational update defaults without changing compiled app identity or signing trust. Packs SHALL be selected only during installer packaging. One resolved policy SHALL support disabled, manual and automatic checks through direct manifest, GitHub Releases and GitLab Releases sources; installation remains explicit.

#### Scenario: Invalid Pack or Source Switch

- **WHEN** installer packaging receives an invalid organization pack or an operator switches a named update source
- **THEN** invalid packs fail packaging without modifying input artifacts or installed application data, invalid source configuration preserves the last valid configuration with a visible error, source switching preserves signing keys, and disabled policy performs no update network activity.

#### Scenario: Public App and Plugin Releases

- **WHEN** an organization declares app/plugin release sources
- **THEN** public sources work without account login, compatible signed app assets are verified, and direct verified plugin installation retains mutation/cleanup ownership and explicit restart behavior without a staging/consent UI.
