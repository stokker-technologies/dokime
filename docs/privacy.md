# Privacy and Data Flows

The host opens an explicitly configured local storage root. SQLite evidence,
Arrow sample payloads, frozen schemas, plugin metadata, settings and report bytes
stay there. An occupied or incompatible root is refused; it is never silently reset.

## Credentials and Access

Secret values and TLS private keys use the operating system credential store.
SQLite retains metadata and protected-slot references. Secret transfer files are
bounded Base64 JSON, not encrypted archives. Pairings contain credentials and must
be handled as secrets. Remote client services use the configured TLS listener,
pinned trust and bearer authorization; clients never access the data directory.

## Plugins and Network Flows

Managed plugins execute trusted installed code in owned child processes. Remote
plugins are user-owned endpoints. They are not sandboxed. Invocation context limits
host operations by their admitted requirements and execution scope; it does not
remove a plugin process's operating-system authority. See [Plugin Security](plugin-security.md).

## Diagnostics

The library ServiceManager owns bounded logs, trace events and diagnostics archives.
Product contributions live in `crates/dokime-runtime-next/src/services/status.rs`.
Registered secret values and sensitive field names are redacted. Product preferences
and credential values are excluded from archive entries. Review diagnostics before
sharing: arbitrary plugin text may include information meaningful to its author.

## Retention

Removing a plugin entry preserves execution evidence. Published reports retain
frozen documents and seals. Explicit lifecycle/deletion operations own retained-data
changes; backing up the root and credential material remains an operator decision.
