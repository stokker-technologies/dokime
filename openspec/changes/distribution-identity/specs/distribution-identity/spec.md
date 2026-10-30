# Distribution Identity

## ADDED Requirements

### Requirement: Install-Time Distribution Identity

A distribution SHALL declare its identifier in the installer manifest; packaging SHALL install it as the distribution identity, and every application directory SHALL derive from it, falling back to the compiled identifier when none is declared.

#### Scenario: Separate Branded Storage

- **WHEN** a distribution declares `identifier = "com.hilti.dokime"` and is installed beside plain Dokime
- **THEN** its application, data, plugins, logs and webview directories live under `com.hilti.dokime` and plain Dokime keeps `com.stokker.dokime`

#### Scenario: Default Identity

- **WHEN** a distribution declares no identifier
- **THEN** directories derive from the compiled identifier

#### Scenario: Install Side By Side

- **WHEN** a branded distribution is installed
- **THEN** its install folder, uninstall registry key and package name use its identifier, while installer titles, pages, shortcuts and native display names use the resolved organization manifest's display label
