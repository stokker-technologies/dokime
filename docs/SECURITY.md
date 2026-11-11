# Security Policy

## Reporting a Vulnerability

The designated private channel is [GitHub Private Vulnerability Reporting](https://github.com/stokker-technologies/dokime/security/advisories/new). Private reporting is currently disabled; the repository administrator must enable it before this channel accepts reports. Include the affected version, platform, impact and minimal reproduction. Do not include customer evidence, credentials or personal data. Stokker Technologies targets acknowledgement within three business days (Zurich business calendar); this is an acknowledgement target, not a remediation deadline.

## Supported Versions

Security triage covers the latest stable release and the nightly channel. Older releases should be upgraded to the latest stable release. Nightly is a development channel and can contain regressions. Fixes land in dev and are released according to severity; a nightly build is not a substitute for a stable security update.

## Coordinated Disclosure

We agree a disclosure date with the reporter after assessing impact, prepare mitigations and a fix, and publish a GitHub Security Advisory with affected versions, remediation and reporter credit with consent. We communicate material delays and coordinate upstream issues privately. Evidence of active exploitation is escalated immediately.

Where the Cyber Resilience Act reporting obligations apply, Stokker Technologies commits to an early warning within 24 hours and a notification within 72 hours of awareness of an actively exploited vulnerability or severe security incident, through the applicable coordinating CSIRT and ENISA reporting mechanism. These elapsed-hour deadlines are independent of the business-day acknowledgement target. Final reports follow the applicable Article 14 deadlines: vulnerability reports within 14 days after a corrective or mitigating measure becomes available, and incident reports within one month of notification. See [Regulation (EU) 2024/2847, Article 14](https://eur-lex.europa.eu/eli/reg/2024/2847/oj).

## Dependency Checks and Exceptions

`mise run check:rust:supply` runs Cargo Deny (advisories, bans, licenses and sources), Cargo Audit, the production pnpm audit and the locked uv audit. Failures fail CI. The license allowlist selects permissive alternatives where available, including MIT for r-efi; MPL-2.0 is allowed with its file-level obligations and notices. GPL-, AGPL- and LGPL-only dependencies are not allowed.

[`supply-chain-exceptions.toml`](../tools/config/supply-chain-exceptions.toml) is the reviewed exception register. Entries identify the advisory, exact package and version, fixed version when available, dependency path, reason, owner and expiry. Current entries are owned by the orchestrator, blocked on compatible upstream dependency updates, and expire on 2026-11-23 (UTC), except RSA and GLib, which retain their 2026-11-22 expiry. The gate rejects expired entries, new findings and different dependency versions. Workspace git dependencies carry explicit version constraints; permissive license expressions and their justifications live in `tools/config/deny.toml` without dated exceptions. The native tools remain strict; their original exit codes and complete reports are retained under `target/supply/`, while the gate accepts only findings matched by the register. Dependency upgrades are preferred to renewed exceptions. GPL-, AGPL- and LGPL-only licenses have no exception.

`mise run package -- --language rust` generates Rust license texts with Cargo About, production shell notices with pnpm and Python notices from hash-verified wheels in `uv.lock`. `target/notices/THIRD-PARTY-NOTICES` includes the complete Rust workspace and Python lock inventory, including development tools. Packaging includes NOTICE and THIRD-PARTY-NOTICES in distribution archives and wheel license directories, and combines them with the staged installer LICENSE so installed copies retain all notices. Release staging generates target-specific Rust inventories and a CycloneDX JSON for each artifact, with a conservative source inventory alongside Syft's scan of its unpacked contents. Release builds use the pinned Cargo Auditable wrapper without changing the build profile.

## Verifying Releases

Download an installer, CLI archive, cargo-dokime archive, sidecar archive or Python wheel and its release's SHA256SUMS. Check the file's SHA-256 against that inventory, then run `gh attestation verify <downloaded-file> --repo stokker-technologies/dokime --signer-workflow stokker-technologies/dokime/.github/workflows/release.yml` for stable releases, or use `.github/workflows/nightly.yml` for Nightly. Repeat for each artifact you intend to deploy; verification binds the exact bytes to the expected repository and workflow. Review the reported source revision and release tag. Attestations establish provenance, not absence of vulnerabilities or reproducible builds.
