## REMOVED Requirements

### Requirement: Two-Layer Organization Packs

**Reason**: Installer packaging selects one complete pack. Runtime overlays, pending packs and activation are removed by the One Pack Path requirement.
**Migration**: Supply a complete pack at installer packaging. Existing application-folder packs are ignored and preserved; historical reports retain their captured appearance.

## MODIFIED Requirements

### Requirement: Native Installer Assembly
The authoring tool SHALL expose `cargo dokime installer package`, which uses an installer manifest, local or pinned verified release artifacts, and organization assets. Packaging SHALL preserve its input artifacts and SHALL not compile application source. Windows packaging SHALL customize icon resources on a staged GUI executable before signing and integrity recording. Packaging SHALL retain the startup identity and bundled-plugin integrity inventory.

#### Scenario: Product or Company Assembly
- **WHEN** native packaging receives valid local artifacts or a verified exact release payload
- **THEN** it produces the requested Windows NSIS or Linux DEB/RPM output, resolves company branding and cleans ephemeral generated inputs.

#### Scenario: Requested Signing Fails
- **WHEN** explicit signing lacks its external credentials or the signing backend fails
- **THEN** packaging fails without claiming unsigned output is signed or exposing secret material.

#### Scenario: Windows Application Icon
- **WHEN** Windows packaging resolves organization application artwork
- **THEN** the staged GUI executable and installed shortcuts use that artwork, requested signing occurs after resource modification, and the source payload remains unchanged.

#### Scenario: Solid Installer Sidebar
- **WHEN** Windows packaging resolves a valid installer.sidebar_color from the organization pack
- **THEN** generated install and uninstall sidebar bitmaps contain only that solid opaque color, with no authored image required; invalid colors fail validation and omitted settings use the declared installer defaults without inheriting another pack.

#### Scenario: Custom Installer Sidebar Image
- **WHEN** the resolved organization pack supplies installer.sidebar_image
- **THEN** packaging uses that pack-relative image instead of sidebar_color for install and uninstall sidebars, validates its reference and content, and fails on invalid input instead of silently using the color.

### Requirement: Harmonized Authoring and Validation
The authoring tool SHALL group new, build, package, dev and inspect under sidecar. Build SHALL produce a .dokime bundle; package SHALL only assemble existing outputs. Product CI SHALL use the candidate installer command, including in nightly CI. Native packaging SHALL use Tauri/NSIS default compression and SHALL NOT inspect completed installer contents. The authoring tool SHALL NOT expose an installer inspect command.

#### Scenario: Existing Artifact Packaging
- **WHEN** CI supplies matching build artifacts to sidecar package or installer package
- **THEN** packaging consumes those bytes without recompilation and required platform jobs remain enforced.

#### Scenario: Native Installer Defaults
- **WHEN** Windows packaging assembles an NSIS installer
- **THEN** it uses Tauri's default solid LZMA compression while preserving the resolved display label, identifier-based installation and CLI PATH hooks.

#### Scenario: Completed Installer Handoff
- **WHEN** native assembly finishes
- **THEN** packaging returns its outputs without parsing or extracting their contents, and `cargo dokime installer inspect` is unavailable; explicit signing and signature verification remain supported.
