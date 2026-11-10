# Plugin Security

This contract describes installed native plugins. Embedded Python providers run
inside their embedding process and do not gain sidecar process separation.

## Trust Boundary

Any desktop user of the machine can install plugins into their local Dokime
backend through Settings → Plugins. There is no organization approval role or
approved-publisher gate. This does not grant access to another user's backend or
bypass operating-system permissions.

`crates/dokime-runtime/src/plugins/catalog.rs` inspects bundle integrity,
declarations, registrations, operation bindings and schemas within configured
limits, and selects an unambiguous executable for the native target. Admission
validates the combined catalog and required contracts before activation.
`crates/dokime-runtime/src/plugins/store.rs` binds the installation to the
SHA-256 of the inspected bytes and verifies the written copy.
`crates/dokime-runtime/src/runtime/plugins.rs` checks that binding again and
compares the extracted executable's digest and declaration with the bundle.
The supervised handshake expects the frozen bundle registrations.

These checks detect changed bytes and incompatible declarations. They do not
review executable behavior or authenticate its publisher: plugin publisher
signatures and publisher revocation are not implemented.

Disable the plugin's enable switch or use **Delete** in Settings → Plugins.
Changes wait until executions, cleanup and stream draining release the plugin;
**Restart Pending** is not immediate containment. Removal discards the managed
bundle and installation row after stopping it; failed deletion remains retryable.
Saved evidence remains. See [Live Installation Changes](plugins.md#live-installation-changes).

## Isolation

Sidecars are separate native processes with the operating-system user's
authority. Process separation and supervision contain process crashes and
protocol faults; they do not contain malice. A sidecar can access files, devices,
networks and process environment available to that account. There is no per-plugin
filesystem, device, network or environment capability sandbox. Running Dokime
with elevated privileges increases the sidecar's authority too.

The supported API mediates secret lookup by identifier and evidence publication
through host operations and validated publication routes. This is an API boundary,
not an OS access restriction: a malicious native process can attempt direct file
access or network exfiltration outside those routes. The host plugin itself uses
native device and system APIs; its current source is `plugins/sidecars/host/`
(the `plugin-host` package).

`crates/dokime-runtime/src/runtime/plugins/lifecycle.rs` supervises sidecars,
bounds startup and applies the declared restart policy. Without restart credit,
a fault deactivates the plugin; enabled policies permit bounded restart attempts.
Supervision cannot undo arbitrary device or filesystem effects.

The underlying carrier supports WebAssembly guests, but Dokime's installed-plugin
admission rejects them and its lifecycle adapter has no enabled guest backend.
No WebAssembly sandbox protection is claimed.

## Secrets

Use the host's OS keyring through Secret Storage identifiers. Required declarations
are resolved before startup; missing required secrets refuse startup, while
missing optional secrets remain absent. Dynamic access uses
`dokime.secrets.lookup`. There is no plaintext storage fallback. See
[Shared Plugin Secrets](product-guide.md#shared-plugin-secrets) and
`crates/dokime-runtime/src/domains/secrets/`.

Never put secret values in authored configuration, ordinary settings, environment
variables or command-line arguments. The host resolves identifiers and delivers
values through the plugin protocol, including resolved startup secret fields;
this necessarily exposes those values to the receiving plugin's memory.
Identifiers are shared host-wide, not per-plugin authorization scopes; the
References list is not exhaustive because plugins can look up identifiers dynamically.

Authors must redact credentials from logs, errors, driver diagnostics, records,
telemetry, reports and crash attachments. Do not log request headers, credential
URLs or secret-bearing debug values, persist plaintext copies, or forward secrets
to unrelated processes or services. Host secret wrappers redact their debug
representation, but forwarded plugin log lines are not a universal secret filter.
Review vendor-library logging as well as plugin code. Replacement affects future
lookups; deletion or rotation cannot recall values already held by a plugin.

## Threat Model

Assets are execution evidence and its provenance, instrument credentials, devices
and the host machine. The relevant actors are faulty or malicious plugins, an
attacker controlling an update feed or bundle source, and a local user.

| Threat | Implemented Controls | Residual Risks |
| --- | --- | --- |
| Faulty plugin crashes, hangs or sends invalid publications | Separate process, supervised lifecycle and deadlines, schema and contract validation, bounded restart policy and deactivation | Resource exhaustion, interrupted tests and external side effects remain possible; process separation does not guarantee host availability. |
| Malicious plugin steals credentials or alters evidence | Host API validates routes and publications; credentials are stored in the OS keyring | Native account authority permits direct access and exfiltration. Valid schemas cannot establish truthful measurements. Shared secret identifiers are not a per-plugin access policy. |
| Compromised feed or substituted bundle | Bundle inspection, archive and executable digests, installed-byte binding | An attacker supplying both a new bundle and its expected checksum can supply consistent malicious bytes. No plugin publisher authentication or revocation exists. |
| Local user installs an unapproved plugin or changes local state | OS account permissions, installation digest checks, disable and removal controls | No organization allowlist exists. A user controlling the account and its storage can replace artifacts or state; local checks are not protection against that account. |
| Plugin leaks a legitimately retrieved secret | Keyring storage, identifier-based retrieval and redacted host secret debug values | Plugin memory, logs, dumps and driver libraries can leak values; rotation cannot erase copies. |

Approval therefore covers the combined host, plugins, operating-system account
and instrument access. Installing a bundle is a decision to trust native code.

## Private Plugins

Treat company plugins as internally developed software: assign an owner, require
code review, retain a dependency and license inventory, run vulnerability and
secret scans, test failure and cleanup paths, and release through an approved
pipeline. Retain the reviewed source revision, build inputs, exact bundle digest
and matching Dokime version. Once signing exists, use an approved publisher key
with an accountable custodian; that is a future control, not a current requirement
enforced by Dokime.

For an exact organization-pack artifact, store the reviewed `.dokime` bundle
inside the pack and select it explicitly with a top-level `[[plugin]]` `path`
entry. Record its SHA-256 in the organization's release inventory and make the
release pipeline compare that digest before packaging. Preserve the pack and
inventory together under review. See [Native Installers](product-guide.md#native-installers)
and `crates/dokime-packaging/src/installer/manifest.rs`.

The installer manifest has no per-plugin `sha256` pin field today. A local path
alone is not a digest pin; the organization's pipeline must enforce the recorded
digest. Generated distribution metadata and installed-byte checks bind the
packaged bytes afterward. Feed names or a mutable Nightly selection do not replace
that approval, and bundling does not prevent subsequent user-installed updates.

## Proposed Roadmap

These proposals are not implemented or approved commitments. The owner decides
their scope and priority separately.

- **Publisher Signing:** Sign bundle identity and content with customer-owned
  trust roots. This requires a signed-envelope contract, key provisioning and
  rotation, revocation distribution, offline and expiry policy, and explicit
  install, update and already-installed-plugin behavior after revocation.
- **Organization Publisher Allowlist:** Let an organization pack select approved
  publishers. This requires a pack policy schema, an authoritative policy owner,
  enforcement on every installation and update route, and defined handling of
  local overrides and plugins that cease to be approved.
- **Optional WebAssembly Guests:** Admit untrusted logic through a constrained
  guest backend. This requires explicit host imports and filesystem, network,
  device and environment capabilities, resource limits, runtime patch ownership,
  negative permission tests and compatibility rules for served client operations.

Signing would establish origin and integrity, not safe behavior. Guest support
would need a reviewed capability contract before any sandbox claim.

## API Compatibility

Dokime is pre-1.0. Data-shape identities are derived from their declarations;
changed shapes receive changed identities rather than a manually maintained
compatibility version. Runtime admission matches required and provided contracts
exactly. Frozen evidence definitions preserve interpretation of saved runs;
they do not promise that an older executable can run against a newer host.

Plugin crates are not published to a package registry; consumers pin their git
tags. Pin matching Dokime, plugin and carrier revisions, retain lockfiles and
bundle digests, and respect the bundle's `requires_dokime` declaration. A version
requirement alone does not prove protocol or behavioral compatibility.

On a Nightly bump, review API and carrier changes, update the matching dependency
pins, rebuild and inspect bundles, and test startup, operations, evidence and
cleanup against that exact host before deployment. Retain a tested release for
rollback rather than treating mutable Nightly as a compatibility guarantee.

There is no promised deprecation window for these development APIs. Removed
interfaces require consumer migration and rebuild; compatibility shims and old
carrier support must not be assumed. Development storage formats may also change
without a version bump; preserve incompatible storage and use a fresh root as
described in [the Product Guide](product-guide.md).
