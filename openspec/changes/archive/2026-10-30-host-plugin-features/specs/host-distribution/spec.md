# Host Distribution

## ADDED Requirements

### Requirement: Host Plugin Without Runtime Special Case

The runtime SHALL treat the host plugin as an ordinary installed plugin: no compiled-in installation, no host owner kind and no brand-derived name.

#### Scenario: No Host Plugin Installed

- **WHEN** the desktop starts without the host plugin installed
- **THEN** the runtime boots, lists no host plugin and offers no host telemetry

#### Scenario: Host Plugin Installed From A Release

- **WHEN** the host plugin bundle from a Dokime release is installed
- **THEN** it appears under its manifest name and its streams are owned like any other plugin's

### Requirement: Organization Pack Bundled Plugins

Organization installers SHALL bundle the `.dokime` files of the pack's `plugins/` folder, verified at packaging time, and install them on first start under their own names without downgrading newer installed versions.

#### Scenario: Incompatible Bundled Plugin

- **WHEN** a bundle in `plugins/` targets another platform or an incompatible Dokime version
- **THEN** installer packaging fails and names the bundle

#### Scenario: Installer Upgrade With Newer Installed Plugin

- **WHEN** an installer upgrade bundles an older version of a plugin than the installed one
- **THEN** the installed version is kept

