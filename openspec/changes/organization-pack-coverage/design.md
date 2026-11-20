# Installer Packaging

## Owner Decisions

The release and organization installer have one producer: the CLI built with the
kit runs `dokime installer package` with a checked-in default manifest and
`--artifacts`. Packaging never downloads in this path. Delete the Python native
producer, its configuration and tests only after updater and native payload parity
is established. Retain the CI graph and job names; change packaging steps only.
The installer is never executed during this delivery.

## Runtime Plugin Inventory

Plugin bundles exclude the following development and private inputs at every
directory depth. Required launch inputs matching these rules are refused with the
existing actionable error; they are never silently dropped. Build and launch
scripts and their runtime metadata remain ordinary inputs unless excluded.

```text
mise.toml .mise.toml mise.lock .tool-versions
.env .env.*
.git .gitignore .gitattributes .gitmodules .github .gitlab .gitlab-ci.yml
.hg .hgignore .hgtags .svn
test tests __tests__ .pytest_cache .mypy_cache .ruff_cache .cache
.idea .vscode .vs .DS_Store Thumbs.db
.venv __pycache__ _input _output credentials credentials.json
.netrc .pypirc id_rsa id_ed25519 *.pair *.pem *.key
```

Native bundles already select executable, assets and referenced metadata only.
Source bundles keep build and launch code, language manifests and lock files.
Audit the four SDK starters for remaining development-only members.

## Kit Identity and Default Plugins

`distribution.json` remains schema 1 and requires `source_revision`, `channel`
and `plugins` in addition to version, target and files. Build/Package writes the
same `DOKIME_SOURCE_REVISION` and `DOKIME_BUILD_CHANNEL` consumed by executable
build scripts; no second revision/channel inference or old-shape reader exists.
The kit lists default plugin identities from the first-party built bundles. Local
kits store those bundles in `plugins/`; downloaded kits select them from the
verified release plugins archive. Every installer includes defaults unless
explicitly excluded. Additional `[[plugin]]` name/path entries remain supported.

```toml
[installer]
version = "1.2.3"
formats = ["nsis"]
exclude_plugins = ["plugin-obc"]

[dokime]
version = "nightly"

[[plugin]]
path = "plugins/bench.dokime"
```

There is one ZIP regular-member writer: `package/assembly.py::write_file`.
Distribution and release plugin archives delegate permissions and file-type bits
to it, retaining their existing integrity and path checks.

## Release Parity

Before deleting the Python producer, reconcile installer names, product label and
identifier, current-user Windows hooks, installed resource locations, Linux
dependencies and desktop template, Authenticode, updater signatures and metadata,
and release verification expectations. The single updater filename is
`latest.json`, already consumed by release staging and desktop updater endpoints.
Release filenames stay `dokime-<version>-<target>-setup.exe`, `.deb` and `.rpm`.
Organization filenames use the distribution slug. Keep signature verification,
archive bounds and operation deadlines unchanged. Stable Windows publication
requires updater signing. No fallback producer or compatibility alias remains.

## Progress Observer

One small caller-supplied observer receives completed steps and their durations:
Manifest, Kit, Appearance, Plugins, Bundle, Sign and Output. The packaging library
prints nothing. Failures identify the active step after prior completed events.
The CLI writes progress to stderr, paths to stdout, and keeps JSON unchanged and
silent on stderr. Nonterminal output has no color or cursor control. Decimal
numbers use dots without grouping. Kit reports version, short revision, channel,
target, acquisition and verified inventory. Plugin entries distinguish defaults
from additions. Output reports paths and sizes; the final line reports installer
count and total duration. The header uses the CLI's shared build identity when
available, with one call site. Signing notices use the same observer.

## Proof and Ownership

Do not edit `docs/`, CLI version ownership or desktop sources. Report documentation
impact by file and line. Validate this change strictly; keep its task checkboxes
current. Run format, scoped test-target compilation, focused packaging tests,
installer acceptance, affected Python tooling tests and repository checks once at
the end. Shared test contracts additionally require workspace test-target
compilation with desktop `testing`. Never run an installer. End-to-end Windows
packaging uses a local kit only if project Build/Package can produce it; compare
produced names and sizes with published Nightly assets. Hosted Nightly must prove
native Windows/Linux packaging and release verification. Commit locally and never
push from this worktree.
