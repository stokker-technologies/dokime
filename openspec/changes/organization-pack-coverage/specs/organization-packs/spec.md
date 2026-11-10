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

### Requirement: Packaged Pack Is Fixed

An installation packaged with an organization pack SHALL use that pack in place of the bundled default and SHALL NOT allow it to be changed.

#### Scenario: Packaged With a Pack

- **WHEN** an installer is packaged with a pack and installed
- **THEN** the installation shows that pack and Settings offers no pack import, change or removal

#### Scenario: Pack Placed in the Application Folder

- **WHEN** a pack is placed in the application folder of an installation packaged with a pack
- **THEN** it is ignored

#### Scenario: Packaged Without a Pack

- **WHEN** an installer is packaged without a pack
- **THEN** the installation keeps the pack import it has today
