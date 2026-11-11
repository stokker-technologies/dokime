## MODIFIED Requirements

### Requirement: Native Installer Assembly

The `dokime` tool SHALL expose `dokime installer package`, using an installer manifest,
local or pinned verified release artifacts and organization assets. Packaging SHALL preserve
input artifacts without compiling application source. Windows packaging SHALL customize
staged GUI icon resources before signing and recording integrity. Startup identity and
bundled-plugin integrity inventory SHALL remain preserved.

#### Scenario: Product or Company Assembly

- **WHEN** packaging receives valid local artifacts or a verified exact release payload
- **THEN** it produces the requested Windows NSIS or Linux DEB/RPM output, resolves branding and cleans ephemeral generated inputs

#### Scenario: Requested Signing Fails

- **WHEN** explicit signing lacks credentials or its backend fails
- **THEN** packaging fails without claiming unsigned output is signed or exposing secrets

#### Scenario: Windows Application Icon

- **WHEN** packaging resolves application artwork
- **THEN** the staged executable and installed shortcuts use it, signing follows modification and the source payload remains unchanged

#### Scenario: Solid Installer Sidebar

- **WHEN** packaging resolves a valid sidebar color
- **THEN** install/uninstall bitmaps use that opaque color, invalid colors fail and omitted override values inherit defaults

#### Scenario: Custom Installer Sidebar Image

- **WHEN** the pack supplies a sidebar image
- **THEN** packaging uses the validated pack-relative image and refuses invalid input without falling back to the color

### Requirement: Harmonized Authoring and Validation

The `dokime` tool SHALL group bundle, dev and inspect under plugin and project creation
under `new --lang`. Bundle packaging SHALL consume an existing executable. Installer
packaging SHALL consume existing product artifacts. Product CI SHALL use the candidate
tool, including Nightly. Native installers SHALL use Tauri/NSIS default compression without
inspecting completed installer contents. No installer inspect command SHALL exist.

#### Scenario: Existing Artifact Packaging

- **WHEN** CI supplies matching executables or product artifacts to plugin bundle or installer package
- **THEN** packaging consumes those bytes without recompilation and required platform jobs remain enforced

#### Scenario: Native Installer Defaults

- **WHEN** Windows packaging assembles NSIS output
- **THEN** it uses Tauri's default solid LZMA compression and preserves the display label, identifier-based installation and CLI PATH hooks

#### Scenario: Completed Installer Handoff

- **WHEN** assembly finishes
- **THEN** outputs return without installer extraction/inspection, `dokime installer inspect` is unavailable and explicit signing/verification remain supported
