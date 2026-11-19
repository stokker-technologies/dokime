# Organization Pack Coverage

## Why

With an organization pack applied, the built-in product name still appears in
places an operator sees: the first window title, report titles, table labels,
suggested file names and error messages. A pack can also arrive two ways,
through the installer or through Settings, which is one path too many.

## The Rule

| Follows the Pack | Keeps the Dokime Name |
| --- | --- |
| What an operator of that installation sees: the desktop app and its windows, dialogs and notifications, reports, error messages, suggested file names, the installer and its shortcuts | What an author or integrator uses across installations: the `dokime` command and its help, `dokime`, scaffolds, `dokime.v1`, `.pair` and `.dokime` files, `DOKIME_*` variables, SDK packages, schema and operation identifiers, credential store names, the pairing-file folder, upstream attribution |

Install, package and data folder names come from the installer's distribution
identifier, not from the pack. That stays.

## What Changes

- The first window and report title and icon use the resolved label and logo
  from the start, not after the interface initializes; the fallback report
  title uses the captured label.
- Interface text reads the label: the version row in traceability tables, the
  plugin compatibility text and the file picker filter names.
- Suggested file names for diagnostics, support bundles, the secrets export
  and log files derive from the distribution identifier.
- Operator-facing error and status messages name the resolved label. Failures
  before the pack is loaded use the distribution's default label.
- The product name inside report data comes from the captured appearance.
- New remote access certificates are issued under the resolved label;
  existing certificates keep their subject.
- Packaging the installer is the only way to apply a pack. A pack supplied
  there replaces the bundled default. Pack import in Settings, the pending
  and active packs in the application folder and their overlay are removed.

## Locked Decisions

- Executable and process names stay as they are. Renaming them touches
  packaging, desktop discovery and installer hooks together and is not worth
  it now.
- The toolchain and contracts in the second column keep the Dokime name.
- There is one path for a pack: the installer. An installation packaged
  without one shows the built-in appearance.

## Impact

- Desktop shell, interface, runtime messages, report rendering, export file
  names and installer packaging.
- No storage or protocol change. Existing reports keep the appearance they
  captured.

## Installer Packaging

The current command is `dokime installer package --manifest installer.toml --output dist`.
`--artifacts <directory>` selects an extracted local distribution kit and prevents
release acquisition, including for named plugins; local named plugins live under
`plugins/<name>.dokime`. `--plugin <path>` is repeatable and accepts current gRPC
`.dokime` bundles or plugin ZIP/TAR.GZ archives. `--format nsis,deb,rpm`, `--target`,
`--sign` and `--json` retain their earlier command semantics.

The manifest schema retains every earlier table and key; no keys are added.
Paths resolve relative to the installer manifest except `plugin.path`, which is
relative to the selected organization folder. A supplied organization folder is a
complete replacement. A custom pack without an explicit update mode disables
updates. Pack identity cannot override the installer distribution identifier.
Executable names remain `dokime` and `dokime-gui`; Windows setup, removal, registry
writes and PATH updates use only the current user. The native packaging boundary
requires NSIS; payload preparation tests do not invoke it or a published release.

```toml
[installer]
version = "1.2.3" # Required exact installer semantic version.
formats = ["nsis"] # Required, native formats only; Linux uses deb/rpm.
identifier = "com.example.testhub" # Optional; defaults to compiled product identity.
organization = "example-pack" # Optional complete appearance pack folder.

[dokime]
version = "nightly" # Optional; exact release version or nightly; defaults to CLI version.
target = "x86_64-pc-windows-msvc" # Optional; defaults to the native target.

[[plugin]] # Optional, repeatable; exactly one of name or path per entry.
name = "example-pack" # Published bundle filename without .dokime.

[[plugin]]
path = "plugins/example-pack.dokime" # Relative to the organization pack folder.

[license] # Optional; both keys required when present; adds to Apache-2.0.
spdx = "LicenseRef-Example"
file = "terms.txt" # UTF-8 text/RTF; confined to the installer manifest folder.

[signing.installer] # Optional; applied only with --sign.
certificate_thumbprint = "0123456789012345678901234567890123456789" # Windows certificate store.
timestamp_url = "https://timestamp.example.test" # Windows HTTPS RFC 3161 endpoint.
# key_id = "0123456789ABCDEF" # Linux GPG agent key; replaces Windows fields.

[signing.updater] # Optional; native Windows; installer/product versions must match.
private_key_file = "updater.key" # Exactly one of these three private-key references.
# private_key_env = "EXAMPLE_UPDATER_KEY" # Base64 key material.
# private_key_path_env = "EXAMPLE_UPDATER_KEY_PATH" # Environment variable holding a path.
password_env = "EXAMPLE_UPDATER_PASSWORD" # Required password environment reference.
artifact_base_url = "https://downloads.example.test" # Required HTTPS publishing base.
```

Acquisition verifies release `SHA256SUMS`, then every distribution inventory member.
Named plugins are selected from `dokime-plugins-<version>-<target>.zip` (Windows) or
`.tar.gz` (Linux), not the removed plugin feed. Updater output uses `release.json`.
The kit no longer carries identity/trust metadata; packaging uses the CLI build's
product configuration for the default identifier and fixed updater verification key.
Signing never reads an organization-supplied trust key.
