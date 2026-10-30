# Organization Installers

## ADDED Requirements

### Requirement: Two-Layer Organization Packs
Dokime SHALL resolve bundled organization defaults with optional folder overrides by merging manifest fields and replacing whole files before validation. Compiled identity SHALL remain outside the organization pack.

#### Scenario: Partial Company Override
- **WHEN** an override changes only a title and light theme file
- **THEN** other manifest fields and files inherit from bundled defaults, and UI/tray/new report captures use the resolved branding.

#### Scenario: Invalid Activation
- **WHEN** a pending resolved pack is invalid
- **THEN** activation reports the error and retains the last valid pack without resetting user data or historical reports.

### Requirement: Native Installer Assembly And Inspection
The authoring tool SHALL expose installer package and inspect commands that use an installer manifest, local or pinned verified release artifacts, and organization assets. Packaging SHALL preserve its input artifacts and SHALL not compile application source. Windows packaging SHALL customize icon resources on a staged GUI executable before signing and integrity recording.

#### Scenario: Product Or Company Assembly
- **WHEN** native packaging receives valid local artifacts or a verified exact release payload
- **THEN** it produces the requested Windows NSIS or Linux DEB/RPM output, resolves company branding, cleans ephemeral generated inputs and validates actual package contents through inspection.

#### Scenario: Requested Signing Fails
- **WHEN** explicit signing lacks its external credentials or the signing backend fails
- **THEN** packaging fails without claiming unsigned output is signed or exposing secret material.

#### Scenario: Windows Application Icon
- **WHEN** Windows packaging resolves organization application artwork
- **THEN** the staged GUI executable and installed shortcuts use that artwork, requested signing occurs after resource modification, and inspection verifies the final packaged bytes while the source payload remains unchanged.

#### Scenario: Solid Installer Sidebar
- **WHEN** Windows packaging resolves a valid installer.sidebar_color from the organization pack
- **THEN** generated install and uninstall sidebar bitmaps contain only that solid opaque color, with no authored image required; invalid colors fail validation and omitted override values inherit the bundled default.

#### Scenario: Custom Installer Sidebar Image
- **WHEN** the resolved organization pack supplies installer.sidebar_image
- **THEN** packaging uses that pack-relative image instead of sidebar_color for install and uninstall sidebars, validates its reference and content, and fails on invalid input instead of silently using the color.

### Requirement: Harmonized Authoring And Validation
The authoring tool SHALL group new, build, package, dev and inspect under sidecar. Build SHALL produce a .dokime bundle; package SHALL only assemble existing outputs. Product CI SHALL use the candidate installer command and inspect the produced packages, including in nightly CI.

#### Scenario: Existing Artifact Packaging
- **WHEN** CI supplies matching build artifacts to sidecar package or installer package
- **THEN** packaging consumes those bytes without recompilation and required platform jobs remain enforced.

