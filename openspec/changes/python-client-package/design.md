# Python Client Package Design

## Ownership
The Dokime product owner is the sole writer on chore/migrate-hub-ci. The parent owns specification activation, the shared native goal, CI oversight and release coordination.

## Contract
The published distribution is dokime-client, the import namespace is dokime_client, and the package lives at packages/dokime-client/src/dokime_client. Protocol names under the private generated wire tree remain dokime/v1 and harmos/host/v1. Existing Rust crate, product, executable and installer identities remain unchanged. Replace old package/import consumers without compatibility aliases.

## Inventory
Package manifests, all authored/generated Python modules, examples and tests move together. Root pyproject.toml and uv.lock select the renamed member/dependency. Authored bindings.py passes the new distribution/module to the existing Harmos exporter and preserves product exports. wheel.py, installer.py, mise.toml, stkr.toml, generated workflows and the two GUI Rust fixture/process-story paths follow the renamed package. README installation/import examples use uv; generated binding freshness and actual installed-wheel payload remain required.

## Proof
Run targeted checks while source is open. After closure, rerun affected Python format/lint/type/generated checks, Product Python wheel/sdist, Python integration/acceptance and Package/Dry Python proof. The canonical cache/job update changes embedded starter templates, so refresh their compiled cargo-dokime consumer and the Tests archive, including existing compile contracts. Rerun affected starter and two Rust GUI test consumers; preserve unaffected application compilation and prior native Rust suites. Verify existing Trusted Publisher expectations as repository stokker-technologies/dokime, workflow managed-release.yml, environment release, without editing account settings. The user reports dokime-client's publisher is already configured; successful publication remains separate proof.

## Required Desktop Repair
Provider System proof exposed plot-settings remount, execution-stop synchronization and shutdown failures. Preserve the same named plot across remounts and assert its actual saved input value, including an empty value rather than timing out on truthiness. The execution-stop story must leave its startup operator prompt unanswered while waiting for the running row; the generic completion helper answers prompts and cannot own that synchronization. Failed graceful backend drainage remains a failure, followed by forced termination and awaited exit of the owned process so diagnostic reports can finish. Individual story deadlines and assertions remain unchanged; Linux native System execution supplies required provider proof.

## CI Job Budget
The user's final decision supersedes the earlier minimum-only policy: ordinary CI jobs default to 120 minutes. Remove unnecessary product overrides rather than retaining shorter historical limits. Exceptions require a concrete hang/deadlock reason and thirty-minute increments. Hub owns the canonical default and pull-request cache-write policy; regenerate from the parent's validated snapshot without provider writes. This does not change individual test deadlines or waive the desktop failures.

## Runner Routing
Reserve local runners only for jobs that actually compile crates, including nested fixture builds, coverage and Cargo publication verification. Route all artifact-only tests, benchmarks, packaging and other noncompiling jobs to matching hosted platforms. Existing stkr.toml job overrides own routing; preserve bootstrap, artifacts, deadlines and required native gates. Reopen affected source before editing and retain unaffected completed proof.

Rust acceptance remains local on both platforms because generated_plugin_contract runs actual scaffold check, build and Tests producers. Unit, integration and system leaves plus benchmark execution consume existing artifacts and use hosted runners. Rust coverage instruments and compiles crates locally; Release Dry, Rehearsal and Tag perform Cargo publication verification locally. Prepare, Publish and Verify remain hosted.
