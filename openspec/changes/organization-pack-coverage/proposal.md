# Organization Pack Coverage

Status: locked by the owner on 2026-11-10; implementation has not started and
waits for the owner's go. Independent of `grpc-plugins`.

## Why

With an organization pack applied, the built-in product name still appears in
places an operator sees: the first window title, report titles, table labels,
suggested file names and error messages. A distribution packaged with a pack
can also have that pack replaced from Settings.

## The Rule

| Follows the Pack | Keeps the Dokime Name |
| --- | --- |
| What an operator of that installation sees: the desktop app and its windows, dialogs and notifications, reports, error messages, suggested file names, the installer and its shortcuts | What an author or integrator uses across installations: the `dokime` command and its help, `cargo dokime`, scaffolds, `dokime.v1`, `.dokime-remote` and `.dokime` files, `DOKIME_*` variables, SDK packages, schema and operation identifiers, credential store names, the client profiles folder, upstream attribution |

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
- A pack supplied when the installer is packaged replaces the bundled default
  and is fixed: that installation offers no way to import, change or remove a
  pack, and ignores a pack placed in its application folder.

## Locked Decisions

- Executable and process names stay as they are. Renaming them touches
  packaging, desktop discovery and installer hooks together and is not worth
  it now.
- The toolchain and contracts in the second column keep the Dokime name.
- An installation packaged without a pack keeps today's pack import.

## Impact

- Desktop shell, interface, runtime messages, report rendering, export file
  names and installer packaging.
- No storage or protocol change. Existing reports keep the appearance they
  captured.
