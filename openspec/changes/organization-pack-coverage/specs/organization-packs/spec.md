## ADDED Requirements

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
