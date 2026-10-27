# Organization Installer Design

## Agreed Contracts
- Two inputs form one resolved pack: bundled defaults and an optional override folder. Missing manifest fields inherit; present scalar/array values replace; tables merge recursively. Files replace at the same normalized relative path. No deletion markers. Resolve before validation; invalid pending activation preserves the last valid pack and reports an error.
- Keep organization.toml, theme/{typography,base,light,dark}.css, fonts and surface/treatment logos. Asset references are pack-root-relative. Add explicit application/tray/installer artwork roles without a second branding format.
- Generated theme/icon/config artifacts are ephemeral assembly inputs. Final installers and verification outputs persist. Preserve required source-generated assets only where current compilation needs them until consumers are migrated in this change.
- Compiled application identifier, storage slug and deep-link scheme belong to build configuration, not runtime packs. Display name/title/logo remain organization values. Do not silently change storage roots or discard data.
- UI, tray and newly captured reports use the resolved pack. Historical captured reports retain their embedded appearance. Fonts/assets in reports remain self-contained.
- Windows packaging replaces the staged GUI executable's PE icon resources with resolved app.artwork.application and makes installed shortcuts use that icon. Preserve compiled application identity and the original verified payload. Apply resource changes before executable signing, then hash the final staged bytes for inspection; never preserve or claim an upstream signature invalidated by patching. Requested signing covers the modified executable and final installer, with the existing fail-closed credential policy. Application artwork also supplies the runtime window and Linux launcher; installer artwork supplies the setup executable icon. No duplicate NSIS template.
- The resolved organization.toml owns [installer].sidebar_color as an opaque #RRGGBB color and optional sidebar_image as a pack-root-relative image path. Bundled defaults supply the color and folder overrides may replace either field under the existing merge rules. A supplied image takes precedence over the color; otherwise Windows packaging generates an ephemeral solid-color NSIS sidebar bitmap. Use the result for install and uninstall sidebars without CSS evaluation or a duplicate NSIS template. Validate colors and image references/content before assembly; invalid images fail rather than silently falling back. This setting has no effect on DEB/RPM assembly. Replace the current manifest comment prohibiting all color values when adding the native installer setting.
- installer.toml owns distribution version, organization path, formats, exact Dokime release/target and signing references. Paths resolve relative to the owning manifest.
- installer package accepts local build artifacts (no compilation) or downloads the exact release payload and verifies integrity. Export a stable payload contract for downstream consumers; do not unpack and mutate an existing installer as the source.
- The payload includes its release's complete default organization and hashes. Packaging resolves company inputs over that base, never over the packaging tool's own compiled defaults. The resolved pack becomes the installation's one default layer.
- Native Windows produces NSIS; native Linux produces DEB/RPM. Reuse available native bundling machinery. Sign only when explicitly requested; fail closed for missing credentials or failed signing. Public settings/key-file references may be in configuration; private material/passwords are external environment/file inputs and never logged or accepted as secret CLI values.
- Installer signing and updater signing remain distinct. Company packs default to disabled update checks; resource customization does not change compiled updater trust. No company in-app update trust expansion.
- installer inspect reads finished package contents without installing/executing the payload, validates embedded distribution metadata against actual files, and supports human/JSON results. Native extraction prerequisites are explicit.
- sidecar new/build/package/dev/inspect replaces the old top-level leaves, migrating all consumers. Ordinary build compiles and packages; package assembles prebuilt bytes. Verify Harmos naming against local source before implementation.

Sidebar images accept PNG, resize to the native 164×314 bitmap, and composite transparency over sidebar_color. Omitted installer settings default to #1B1915 so previously built payloads remain usable.

Windows resource replacement must retain Tauri's application icon resource identity. Runtime organization branding must set both the small window icon and the large taskbar icon; setting only the small icon is insufficient. Validate the native resource lookup and large-icon behavior rather than relying solely on package hashes or title-bar appearance.

## Ownership
Organization examples in fixtures, fonts and assets use Stokker Technologies. Preserve authentic third-party font attribution and Dokime product identity. Customer names do not belong in tracked files or branch history.

Organization owner: organization assets/schema and build identity split, domain/runtime brand/config/capture, GUI organization/tray/appearance, their tests and organization preparation tooling.
Installer owner: dokime-packaging installer implementation and cargo-dokime installer leaves, native staging/download/inspection/signing and targeted tests.
Integration owner: OpenSpec, root manifests/lock, existing sidecar command grouping and all caller migrations, CI/build exports/package tasks/release plumbing, documentation, integration and final proof.
Owners coordinate shared type interfaces before edits. Cargo manifest dependency requests go to integration owner; no competing lockfile writes.

## Delivery
The combined Dokime baseline consumes published Harmos 0.1.5 and preserves production deadlines and durable acknowledgement. Installer validation uses native packaging checks and generated-artifact inspection. The user explicitly excludes installing the installer, an installed shortcut/uninstaller walkthrough and temporary Windows account creation from the required validation scope.

All organization-installer changes are consolidated into chore/migrate-hub-ci and deliver through the existing protected PR 7 into dev. Only targeted proof runs until all combined implementation items close and writers hand off; repair failed jobs and affected dependents from the existing all-platform simulation and hosted CI. Preserve native required checks and provider auto-merge. Finish the specification after its remaining validation tasks are complete, then retire the superseded feature worktree after verifying remote preservation.

