# Support

## Channels and Response Targets

Report product defects and usage questions through [GitHub Issues](https://github.com/stokker-technologies/dokime/issues).
Report vulnerabilities privately through the channel in [SECURITY.md](../SECURITY.md)
(pending publication). Do not post vulnerabilities or private evidence in public issues.
Commercial support contact: **Pending owner confirmation**. A named primary contact,
backup and staffed calendar remain to be confirmed; no commercial SLA is offered here.

These acknowledgement targets are **proposed, pending owner confirmation**.
Acknowledgement means a human response and initial triage, not a fix deadline.
Proposed business days are Monday–Friday, excluding public holidays at the
confirmed support location; the owner must publish that location and coverage hours.

| Severity | Example | Proposed Acknowledgement |
| --- | --- | --- |
| Critical | Active exploitation or widespread loss of evidence | 1 business day |
| High | Core workflow unavailable without a workaround | 2 business days |
| Medium | Impaired workflow with a workaround | 5 business days |
| Low | Usage question or minor defect | 10 business days |

Statutory incident-reporting clocks run in elapsed hours independently of these
targets; see [Compliance](compliance.md). No continuous monitoring or guaranteed
resolution time is implied.

## Releases and End of Life

| Channel | Policy |
| --- | --- |
| Stable | Versioned release for adoption after customer qualification; exact-commit release checks are required. Pre-1.0 does not promise API or storage compatibility. |
| Nightly | Mutable development prerelease from dev. Build/package success permits publication while tests run separately and may fail. No support promise; inspect the run and report before use. |

**Proposed, pending owner confirmation:** only the latest stable version receives
maintenance. Security fixes ship in the next stable release; older versions receive
no backports and reach end of life when replaced. There is no fixed release cadence
or promised fix date. A retirement announcement should identify the replacement,
upgrade constraints and evidence-export guidance. Nightly has no maintenance window.

This development policy does not establish a CRA support period. Before applicable
market placement, the owner must adopt the support-period rationale and obligations
described in [Compliance](compliance.md); replacing a version cannot extinguish them.

## Upgrades and Continuity

Read the [release notes](https://github.com/stokker-technologies/dokime/releases),
[product guide](product-guide.md) and [Python SDK guide](../sdk/python/README.md).
Pin matching application, client and plugin revisions. Export evidence and preserve
backups before upgrading; qualify the new revision on a separate storage root.
Development storage formats may break without a version bump, and no automatic
migration is promised. Keep incompatible existing roots intact and use a fresh root.

The [Apache-2.0 source](../LICENSE), [documented build](../CONTRIBUTING.md#development-setup)
and [exportable evidence](product-guide.md#cli-reference) allow customers to retain
results and arrange independent maintenance. Archive the source revision, lockfiles,
toolchain, build instructions and evidence format documentation alongside exports.
These assets do not guarantee maintainer availability, replacement expertise,
continued signing credentials or byte-identical independent builds.
