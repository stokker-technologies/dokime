# Design

## Ownership And Dependencies

Own only dokime.wt/remote-streamline and its feat/remote-streamline branch through PR2 integration. The later single release branch is release/v0.1.0 in this same workspace. Preserve all existing commits. No drop worktree or Hub edits are authorized here.

Upstream change release-v0-1-1 in harmos.wt/release-rehearsal-render must verify Harmos 0.1.1 publication first. Root Cargo.toml owns harmos, harmos-domain, harmos-packaging and harmos-sdk declarations. Inspect every tracked lock, starter template and bootstrap source for Git pins or integration patches; remove obsolete overrides and verify Cargo resolves the published registry graph.

Harmos marker transfer must use the published Hub CLI's same-change, same-worktree specification rebind with the expected old branch. The human rejected direct marker editing and destructive cancellation. An exact published revision may be bootstrapped locally in the development profile for that authoring operation when the installed CLI lacks it; this grants no release-renderer readiness. Preserve both workspaces; Dokime registry adoption remains dependent on actual Harmos publication.

Replacement AGENTS guidance supersedes the handoff's release-to-main/backmerge flow. Both products consume the verified published Hub generator implementing Release / Prepare, a release review into dev with Release / Rehearsal, squash integration, Release / Tag on that dev commit, a main fast-forward and tag publication/verification. Replace the former main-promotion paths in release/registry.py and release/run.py, their workflows and matching scaffold copies together. Preserve candidates, signing and protected checks; finish retiring the obsolete tracked changelog through its owner before declaring source complete.

The human approved replacing Harmos main's legacy review/21-check requirements with release-identity-only fast-forwards and equivalent first-release main protections for Dokime, retaining signed commits and force-push/deletion protections. Dev retains the complete rendered graph, Release / Dry, Release / Rehearsal and strict reviews. Each product uses its own long-lived, single-repository fine-grained PAT as repository secret RELEASE_TOKEN with the organization's maximum lifetime. Public rehearsal-check reads require live proof. Both identities are riccardostokker; each product's existing single-member team is its sole stable-tag bypass, and main admits only that user. The release environment permits protected dev and v* tags with its reviewer gate. Squash integration deletes source branches while main/dev remain protected. Never substitute the broad local OAuth credential.

```yaml
dev:
  required_checks: complete_rendered_pr_graph_including_release_dry_and_rehearsal
  require_up_to_date: true
  require_signed_commits: true
  force_pushes: false
  deletion: false
main:
  update_identity: approved_RELEASE_TOKEN_identity
  advancement: tagged_dev_commit_fast_forward_only
  required_reviews: false
  required_checks: []
  require_signed_commits: true
  force_pushes: false
  deletion: false
stable_tags:
  creation_identity: approved_RELEASE_TOKEN_identity
  replacement: false
  deletion: false
```

Keep auto-merge enabled. Enable native merged-task source deletion only after retained dev/main references are protected. Actual token identity and the provider representation of exclusive tag/main writes remain unresolved; do not invent an actor id, reuse the broad role-wide tag bypass as proof of identity exclusivity, or expose credentials in source.

Require separate producer readiness evidence for marker rebinding and the release renderer. The upstream owner reports repairs are needed so rehearsal refuses missing candidate-tree evidence and Tag checks main ancestry before creating a tag or publishing Rust. A rebind-ready binary does not authorize generation or final validation with an unrepaired release engine.

The existing tools/python/src/dokime_tools/ci/bootstrap.py registry contract requires all four dependencies to declare exactly =0.1.1, with one matching public registry package each in Cargo.lock. Retain supported explicit local overrides and their tests; they are authoring behavior, not active integration patches. The only tracked project Cargo.lock is the root lock.

## Inherited Source Boundaries

These inherited ownership boundaries scope the complete PR2 source accounting. Verify moved paths against the current tree; the table records the required cutover, not a claim that every item is complete.

| Domain | Existing Owners / Paths | Required Cutover |
| --- | --- | --- |
| Pure contracts | `crates/dokime-domain/src/model/`, `operations/execution/`, `operations/prompts/`, content/settings operations, transactions/state | Lifecycle, queue admission, seal/export/remote settings contracts and registries; no I/O in domain |
| Runtime | `crates/dokime-runtime/src/domains/execution/`, `domains/exports/`, `domains/prompts/`, `transactions/`, `state/`, `runtime/command/`, `runtime/storage/`, shutdown/startup owners | Durable per-execution Start, Stop-and-seal, ordered cycle continuation and global serialized scheduling; replace public batch/request-key orchestration; durable sealing/recovery and bounded shared exporter |
| Server | `crates/dokime-server/src/host/`, `grpc/`, configuration/access owners | Listener lifecycle, credentials, certificate persistence, authorization simplification, remote operations/export/prompt adapters |
| Rust client | `crates/dokime-client/src/`, connection/profile/credential and download owners | Consistent connection policy, lifecycle/prompt/export handles, uncertainty and transfer guarantees |
| CLI | `crates/dokime/src/commands/{cycle,execution,operation,auth}/`, `connection.rs`, command catalog | Harmonized StartExecution/StopExecution/SealCycle commands, no cycle start/stop or execution seal, remove caller Start request keys and compound start shortcuts; retain prompt/export, input/output files, dotenv and error/exit behavior |
| Desktop | `crates/dokime-gui/src/features/{cycles-home,cycle,settings}/`, routes, native API adapters; `src-tauri/src/{native_session,connection}/` | Explicit cycle destination, active/history presentation, listener settings, one-time key dialog/certificate download, native-owned files and secrets |
| Python | `packages/dokime-client/src/dokime_client/client/`, public facade, generated wire/types, `pyproject.toml` | Parity, explicit dotenv support, prompt interaction, listening/final drain, export and credential handling |
| Generated bindings | `crates/dokime-packaging/`, domain generation inputs, generated Rust/Python/TypeScript consumers | Regenerate from canonical declarations; no independent wire shapes |
| OBC | `plugins/obc-plugin/src/{domain,operations,suites,prompts}/`, plugin/runtime tests and package assembly | Deterministic no-prompt and confirmation scenarios; real public operation effects |
| Examples / Guidance | `packages/dokime-client/examples/remote/`, root README, package/plugin READMEs, `.agents/` contracts, legacy `packages/dokime/examples/remote_control.py` and `packages/dokime/examples/client/poll_run.py` | Beginner guides, twelve scripts, published-package lock, remove overlap and stale commands; update affected canonical/generated guidance |
| Validation / Delivery Tooling | Crate/package tests, `crates/dokime-gui/e2e/`, `tools/python/src/dokime_tools/{build,package,test,release}/`, mise tasks, release/workflow generation owners | Execute actual scripts, native GUI flows, release packaging/installation verification; Python Build stages canonical examples and Python Package assembles them without rebuilding; change rendered files only through their owner |

## Source Inventory And Phase

Current phase is implementing, waiting for upstream authoring capabilities and registry publication. Source inventory includes root and nested Cargo manifests/locks, tools/python dependency bootstrap if affected, cargo-dokime starter templates if affected, README dependency claims, release/version policy, canonical replacement of legacy release helpers/workflows/changelog/provenance consumers, license PR3, OpenSpec lifecycle files and existing PR2 source. The owner-generated release preparation aligns workspace path requirements and carriers; notes come from git-cliff at tag time, with no tracked changelog or release manifest in the final flow. No new feature-drop behavior, API redesign, docs expansion or rustdoc is included.

The old release-manifest consumer census finds generic release/run.py, release/registry.py and package/run.py plus their cargo-dokime scaffold copies; include them in owner-generated adoption. Existing documentation-tool copies require disposition through their generator without authoring product docs. No tutorial, binding-preparation or native desktop script directly consumes release.json. Harmos also has a README download claim to update when its marker can be rebound. Preserve unrelated product package stamps and fixture identities rather than deleting JSON files by name.

Preserve the inherited remote-streamline ACCEPT-01 through ACCEPT-14 without reducing scope: idempotent admission and continuation; durable execution/cycle sealing; shutdown and crash recovery without hardware replay; atomic operation input/output; GUI-managed TLS credentials and rotation; listener rollback; credential precedence/redaction; complete evidence exports; verified bounded atomic downloads; all twelve native/Python scripts; prompts; source-checkout-free candidate installation/tutorial; native GUI lifecycle/security/history; complete API cutover and generated contract parity. Recover details from the preserved legacy .stkr/specifications/remote-streamline/specification.md and execution.md into tracked acceptance before claiming source complete. Do not delete these legacy files or their references.

Before validation, inspect all inventory items, confirm source complete and lift no other owner's holds. Regenerate affected clients only if supported source generation requires it. Coordinate with the orchestrator before any full local Dokime lane because drop workers share the repository's resources.

## Inherited Functional Acceptance

| ID | Required Result And Verification |
| --- | --- |
| ACCEPT-01 | Create two Active cycles and append independently; only one execution runs server-wide. Starting the same execution concurrently, while queued/running, after a lost reply or after completion returns the same durable admission/result without duplicate dispatch or caller request keys; conflicting options and unknown IDs fail. Explicit Start begins work; enabled continuation advances in cycle order only after completion and durable seal. Explicit Stop halts continuation; failure follows declared policy; appending to an idle cycle does not dispatch; reaching the end leaves it open. Exercise admission/sealing races and server-owned continuation without a client scheduling loop. |
| ACCEPT-02 | Completed executions automatically seal while their cycle stays open. StopExecution halts continuation and durably finalizes only the selected execution with a truthful outcome, preserving siblings. SealCycle closes admission immediately, stops/cancels unfinished members and waits until every member and the cycle are durably sealed. Repeating Stop/finalization is harmless; late admission is refused. Persistence failure never reports Sealed. Lifecycle seal state remains distinct from Passed/Failed/Cancelled/Interrupted outcomes. |
| ACCEPT-03 | Server SIGINT and graceful shutdown seal all Active cycles/executions before successful exit. Closing/reopening a GUI attached to a surviving server preserves its Active cycles/work; closing its GUI-managed server seals everything and later shows history. Crash recovery preserves evidence, settles old cycles and never replays hardware; refuse new admission until recovery finishes. Client disconnect alone does not shut down the server. |
| ACCEPT-04 | File/inline operation inputs validate before dispatch; output conflict prevents any device effect; atomic replacement and post-dispatch write failures preserve existing data; uncertainty is nonzero and never retries operations. Test streamed operation output once its shape is locked. |
| ACCEPT-05 | GUI enables persistent TLS access, generates/copies/downloads one-time key and public certificate; restarted clients connect with retained credentials; rotation rejects old-key requests and closes old streams without cancelling execution. Test authentication-off and explicit plaintext independently. |
| ACCEPT-06 | Listener configuration rollback restores the previous working listener on failure; successful change disconnects remote clients while active cycles/executions remain. Native managed-session shutdown still owns server teardown. |
| ACCEPT-07 | CLI and Python credential precedence and explicit-file failures match exactly; dotenv never overwrites environment or searches parents; logs/exports contain no key/private-key/secret values. |
| ACCEPT-08 | Export one sealed execution from an Active cycle and a sealed cycle with all members; SQL reads reproduce captured inputs, record/sample counts, units and outcomes; artifact/report checksums verify; missing required evidence fails. Restart does not prevent retrieval. |
| ACCEPT-09 | Transfer failure, invalid checksum, existing destination, overwrite and retry preserve atomicity and never rerun execution; bounded export/download behavior is exercised with nontrivial stream/file volume. |
| ACCEPT-10 | Actual twelve scripts execute against the built deterministic OBC fixture on supported native platforms: sh on Linux, ps1/bat on Windows, Python on both. Each language uses matching editable global parameters and grouping order; Python has no CLI parser, native scripts call dokime from PATH without executable overrides or generic argument forwarding, and no example manages Start request keys. Retain exact --id-only capture. Assert device effects, observation/final drain, cancellation, prompt response and bundle contents rather than exit status alone. |
| ACCEPT-11 | Confirmation prompt discovery/answer targets explicit execution and prompt identities; rejected/stale/duplicate answers preserve existing prompt semantics; test cancellation while waiting. |
| ACCEPT-12 | Beginner can follow exact candidate installation -> GUI setup -> second-PC CLI/uv workflows using the bundled matching wheel -> SQLite inspection without source checkout, hidden credentials or unexplained substitutions. Check README links, package/artifact versions, script readability, conventions and platform commands. Public release availability remains separate. No claimed success from source inspection alone. |
| ACCEPT-13 | Changed native GUI cycle/history/security/prompt flows pass the native desktop story and qualitative usability review: execution Stop invokes durable StopExecution, cycle Seal invokes SealCycle and removes it from Active only after successful durability while retaining history. Cover GUI reconnection versus managed-server shutdown, explicit selection, visible Sealing/waiting/disconnected states, accessible controls and no secret leakage. Browser mocks do not establish native success. |
| ACCEPT-14 | CLI execution start/stop and cycle seal map to StartExecution/StopExecution/SealCycle shared by GUI and Python. Blocking CLI execution run and Python execution.run compose idempotent Start plus durable completion and requested listener drain without a separate RunExecution primitive. Remove cycle start/run/stop and batch/request-key orchestration, separate public execution seal, duplicate finalization buttons and superseded examples/authorization/tests/guidance. Local credentials/configuration/output do not acquire artificial server operations. Generated bindings match canonical contracts; the full source inventory is accounted for. |

## Acceptance And Authority

Run env -u TMPDIR mise run check before every push and sign all commits. Required native graph includes Publish / Dry with registry-only dependencies; preserve all mandatory checks despite deferred additional project census. Make PR2 ready with native auto-merge after final source gates. Announce its verified integration so the feature drop can target dev.

Integrate license PR3, prepare the release review into dev with the migrated tooling, and pass exact-candidate rehearsal plus all Linux/Windows jobs. Verify the squash-merged dev tree matches the rehearsed review head. Notify the user before enabling its merge, because the resulting dev push automatically runs Release / Tag. Tag creates the stable tag, publishes Rust crates and fast-forwards main; the tag pipeline publishes assets/Python and verifies each platform. Verify main equals the tag commit and that commit is incorporated into dev; no separate review into main or backmerge. Approve authorized protected release deployments and verify remote registries/assets/checksums/native install jobs. Release authority persists from the handoff. No recurring monitoring or new benchmark program. The inherited specification explicitly opts into applicable existing capture/storage/report/transport/shutdown benchmarks; retain those tasks and report their limits after functional validation. Fresh Windows bootstrap-image exercise was explicitly excluded; configured native Windows CI remains required.

Installed-desktop proof and actual key/certificate save dialogs remain acceptance requirements; a WebDriver picker substitution does not prove the native picker. Reconcile current native reports with these obligations after source completion and coordinate local lane ownership before executing missing proof.

Stable Windows signing requires both TAURI_SIGNING_PRIVATE_KEY and TAURI_SIGNING_PRIVATE_KEY_PASSWORD in the protected release environment. The human explicitly authorized generating a new encrypted updater key and provisioning both secrets with gh. Dokime has no published release; update the candidate's configured updater public key to match this initial release key. Retain a private local backup outside Git, verify the key pair with the existing signature verifier, and preserve protected native signed-package verification. This supersedes the earlier instruction to preserve the unavailable old key; do not weaken signing.

PR3's earlier native Windows unit job timed out in the frozen-cycle crash-recovery story; PR2's newer tree relocates cycle_restart to acceptance and has passing Windows unit/acceptance lanes. Do not waive PR3's failure: update it against integrated PR2 and require the resulting native graph before merging. Inspect any remaining failure in its current source before repair.
