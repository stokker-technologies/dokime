# Product Handoff Integration

## Source Ownership

Continue in dokime.wt/hub-ci on chore/migrate-hub-ci. The existing Dokime subagent owns product source and conflict resolution; the orchestrator owns this integration brief and cross-product dependencies. Do not create another checkout or publish intermediate merges.

The preserved migration candidate is the signed local commit 44719b2. Product inputs are origin/feat/drop-2026-10-21 at b837090 and origin/feat/spec-completion at 13c0fd6. Neither product branch subsumes the other. Resolve their full source inventory and maintained OpenSpec contracts before application edits; branding alone is not the delivery unit.

## Integration Policy

Use local merges and explicit conflict resolution to preserve all intended product behavior. Do not select an entire side mechanically, infer that a newer timestamp supersedes an unrelated change, or discard dirty work. Preserve source provenance in Git. Escalate genuinely contradictory public contracts or data-loss decisions; routine adaptation to already approved contracts remains implementation work.

The branding variants and larger sidebar presentation must be reconciled with the later organization/logos ownership, not restored by recreating a superseded branding tree. Inspect manifests, generated assets, GUI consumers, runtime report assets and packaged bytes together. The final source and app must visibly represent the intended new identity.

## Branch Inventory And Contract Resolution

The common migration baseline is dev 5e9a55b. The preserved migration contains 29 commits beyond that baseline; drop contributes 280 and specification-completion contributes 336. The two product branches share ae52bc8 and diverge by eight drop-only and 64 completion-only commits. The original dev-to-drop comparison touches 2,904 paths, including renamed and regenerated files. This is a repository-wide integration, not an asset replacement.

The completed inventory reads both product branches' drop proposal, design and numbered tasks, including the absorbed Organization Pack and Shared Secret Storage contracts. Specification-completion already absorbed those separate changes; do not resurrect their competing source ownership. Historical source-completion and slice-proof claims describe their original branches, not acceptance of this combined tree.

| Owner | Product Work To Integrate | Migration Contract To Retain |
|---|---|---|
| Domain And Runtime | Catalog uniqueness, event-driven plots/watchers, bounded file selection/readiness, telemetry and artifact durability, frozen continuation, report fidelity, notes and shutdown | Cancellation settlement fix, portable archive fixture paths, native feature proofs and honest deadlines |
| Entry Library And Shipped Interfaces | Entry-library facade, runtime Error identity, plugin-sidecar feature and re-exports; newer CLI/server operations and consumers | Published library package identities, non-publishable executable owners, unchanged executable names and raw-asset ownership |
| Organization And Appearance | organization/organization.toml, four surface-aware logos, larger sidebar/wordmark, next-start activation, validated legacy import, origin-scoped update sources and compiled trust | One authored source and generated asset chain, self-contained published runtime resources and freshness checks; no silent data reset |
| Shared Secrets | OS-keyring-backed shared names, immutable slot generations, explicit adoption, masked host-attributed settings and bounded authenticated encrypted transfer | Preserve stored values and separate app authentication; no secret values in logs, packs or generated artifacts |
| GUI And Native Bridge | Path-based plugin/report transfer, plot/report parity, settings, active destination, remote access, organization and secret workflows | Same complete journeys on Windows/Linux, matched native drivers, elevated WebView2 option forwarding, prompt-navigation repair and visible failure evidence |
| Clients And Bindings | Complete updated Rust/Python/TypeScript catalogs, authoring macros and transport consumers | dokime-client distribution, dokime_client imports and uv-only examples; regenerate bindings from the combined catalog |
| Plugins And Starters | plugin-sidecar consumers, renamed SDK support, compile-time catalog checks and generated client/module staging | Offline standalone mise boundaries, root uv workspace, current managed tooling and no private tool dependency |
| Build And Package | Product fixture staging, wheel/source distribution publication, exact tutorial-wheel lock and explicit relock authoring | Shared native producer, shipping-only Product, optional Tests compatibility configurations, matched Clippy and release-tests; artifact-only consumers |
| CI And Bootstrap | Required native prerequisites for the complete newer workflows | Reviewed Hub 876aab97e97a root/starter generation, hosted noncompiler jobs, declared bootstrap, 120-minute job defaults, protections and cache/cancellation policy |
| Documentation And Specifications | Full feature-drop inventory, organization/secrets guidance, linear examples and published surface documentation | Update current contracts together; historical proof is not final acceptance; optional benchmarks remain deferred |

The eight drop-only commits cover the stkr 0.1.7 render, generated client/fixture staging, fixture metadata and desktop click stability, linear shell examples and device guard exit status, a bounded record-wait test, Python source-distribution publication, and executable publication boundaries. Preserve their intended behavior through the newer owners rather than restoring obsolete Validate tooling or executable targets inside publishable libraries.

Completion-only work includes organization ownership/update policy, shared secret storage and transfer, runtime durability/performance corrections, watcher readiness/rotation, live/frozen plot parity, native workflows, optimized development profiles and later facade/client consolidation. Its explicit data-preservation and compiled-trust contracts remain authoritative. Prior benchmark results stay historical evidence; the user's current optional-benchmark decision governs this delivery.

No unresolved public-contract or data-loss decision was found in this inventory. The newer facade defines the intended product API; migration preservation keeps its published package identities and approved executable split. Existing known-secret adoption and legacy organization import remain explicit and retain original data on invalid input.

Integrate specification-completion first because it carries the later absorbed product ownership, then merge the eight divergent drop changes explicitly. Preserve both inputs in local merge history. Resolve authored functionality by these contracts; regenerate managed root/starter outputs from Hub rather than hand-merging generated bodies. Port staging, packaging and native harness repairs into the resulting owners, remove superseded paths, then regenerate clients and review the complete source inventory before final proof.

Preserve the migration's library API identities, non-publishable executable owners, dokime-client distribution and dokime_client import, uv examples, self-contained published runtime assets with freshness checks, shared native Build with configuration-matched Clippy, optional Tests production, hosted-compatible bootstrap and native Windows/Linux UI coverage. Apply the latest reviewed local Hub override and regenerate root and starter workflows without provider credentials or protection writes.

## Proof And Delivery

Previously completed proof remains useful baseline evidence but does not establish the combined product tree. Stop the owned validation sequence at its current safe terminal boundary; do not interrupt unrelated runner work. During integration run targeted checks only. Close every implementation item and review the complete combined tree before one full applicable local validation. Repair failures and rerun affected lanes without weakening assertions or deadlines.

Update the existing PR only after local proof passes. Verify the pushed SHA, retain provider protections and auto-merge, monitor required native CI, and verify the merge into dev. Release remains subject to the existing authorized publication flow and its prerequisites; no new Hub release is authorized. Do not delete the input branches or historical work as part of conflict resolution.
