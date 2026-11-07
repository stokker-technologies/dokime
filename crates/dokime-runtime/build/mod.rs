/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Storage Format Build Inputs

/* === Modules === */

mod source;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{env, path::PathBuf};

// Third Party Imports
use harmos::schema::{Described, graph::Schema};
use sha2::{Digest, Sha256};

// Local Imports

/* === Constants === */

// Explicit Stored Encodings; No Directory Walk Or Whole-Repository Hash.
const SOURCES: &[(&str, &str)] = &[
    (
        "dokime-runtime/execution/context/captured/native/appearance.rs",
        include_str!("../src/execution/context/captured/native/appearance.rs"),
    ),
    ("dokime-runtime/execution/context/captured/native/context.rs", include_str!("../src/execution/context/captured/native/context.rs")),
    ("dokime-runtime/execution/context/captured/native/records.rs", include_str!("../src/execution/context/captured/native/records.rs")),
    ("dokime-runtime/execution/context/captured/native/source.rs", include_str!("../src/execution/context/captured/native/source.rs")),
    (
        "dokime-runtime/execution/context/captured/native/telemetry.rs",
        include_str!("../src/execution/context/captured/native/telemetry.rs"),
    ),
    ("dokime-runtime/storage/reports.rs", include_str!("../src/storage/reports.rs")),
    ("dokime-runtime/storage/sqlite/history/work.rs", include_str!("../src/storage/sqlite/history/work.rs")),
    ("dokime-runtime/runtime/storage/record_capture/source.rs", include_str!("../src/runtime/storage/record_capture/source.rs")),
    ("dokime-runtime/runtime/query/snapshots.rs", include_str!("../src/runtime/query/snapshots.rs")),
    ("dokime-runtime/runtime/query/tokens.rs", include_str!("../src/runtime/query/tokens.rs")),
    ("dokime-runtime/storage/format.rs", include_str!("../src/storage/format.rs")),
    ("dokime-runtime/storage/objects/mod.rs", include_str!("../src/storage/objects/mod.rs")),
    ("dokime-runtime/config/storage.rs", include_str!("../src/config/storage.rs")),
    ("dokime-runtime/config/settings.rs", include_str!("../src/config/settings.rs")),
    ("dokime-runtime/plugins/model.rs", include_str!("../src/plugins/model.rs")),
    ("dokime-runtime/state/encoding/mod.rs", include_str!("../src/state/encoding/mod.rs")),
    ("dokime-runtime/state/encoding/catalog.rs", include_str!("../src/state/encoding/catalog.rs")),
    ("dokime-runtime/state/encoding/executions.rs", include_str!("../src/state/encoding/executions.rs")),
    ("dokime-runtime/state/ownership.rs", include_str!("../src/state/ownership.rs")),
    ("dokime-runtime/state/record_history.rs", include_str!("../src/state/record_history.rs")),
    ("dokime-runtime/state/shared.rs", include_str!("../src/state/shared.rs")),
    ("dokime-runtime/state/shared/map.rs", include_str!("../src/state/shared/map.rs")),
    ("dokime-runtime/storage/objects/annotations.rs", include_str!("../src/storage/objects/annotations.rs")),
    ("dokime-runtime/storage/objects/artifacts.rs", include_str!("../src/storage/objects/artifacts.rs")),
    ("dokime-runtime/storage/objects/case.rs", include_str!("../src/storage/objects/case.rs")),
    ("dokime-runtime/storage/objects/cycle.rs", include_str!("../src/storage/objects/cycle.rs")),
    ("dokime-runtime/storage/objects/execution.rs", include_str!("../src/storage/objects/execution.rs")),
    ("dokime-runtime/storage/objects/metadata.rs", include_str!("../src/storage/objects/metadata.rs")),
    ("dokime-runtime/storage/objects/presentation.rs", include_str!("../src/storage/objects/presentation.rs")),
    ("dokime-runtime/storage/objects/suite.rs", include_str!("../src/storage/objects/suite.rs")),
    ("dokime-runtime/storage/objects/templates/analysis.rs", include_str!("../src/storage/objects/templates/analysis.rs")),
    ("dokime-runtime/storage/objects/templates/conditions.rs", include_str!("../src/storage/objects/templates/conditions.rs")),
    ("dokime-runtime/storage/objects/templates/documents.rs", include_str!("../src/storage/objects/templates/documents.rs")),
    ("dokime-runtime/storage/objects/templates/expressions.rs", include_str!("../src/storage/objects/templates/expressions.rs")),
    ("dokime-runtime/storage/objects/templates/layout.rs", include_str!("../src/storage/objects/templates/layout.rs")),
    ("dokime-runtime/storage/objects/templates/mod.rs", include_str!("../src/storage/objects/templates/mod.rs")),
    ("dokime-runtime/storage/objects/templates/prompts.rs", include_str!("../src/storage/objects/templates/prompts.rs")),
    ("dokime-runtime/storage/objects/templates/suite.rs", include_str!("../src/storage/objects/templates/suite.rs")),
    ("dokime-runtime/storage/records/mod.rs", include_str!("../src/storage/records/mod.rs")),
    ("dokime-runtime/storage/requests.rs", include_str!("../src/storage/requests.rs")),
    ("dokime-runtime/storage/values.rs", include_str!("../src/storage/values.rs")),
    ("dokime-runtime/storage/telemetry/typed.rs", include_str!("../src/storage/telemetry/typed.rs")),
    ("dokime-runtime/storage/telemetry/sample.rs", include_str!("../src/storage/telemetry/sample.rs")),
    ("dokime-runtime/storage/telemetry/encode.rs", include_str!("../src/storage/telemetry/encode.rs")),
    ("dokime-runtime/storage/ipc.rs", include_str!("../src/storage/ipc.rs")),
    ("dokime-runtime/storage/ipc/schema.rs", include_str!("../src/storage/ipc/schema.rs")),
    ("dokime-runtime/storage/ipc/values.rs", include_str!("../src/storage/ipc/values.rs")),
    ("dokime-runtime/storage/sqlite/catalog.rs", include_str!("../src/storage/sqlite/catalog.rs")),
    ("dokime-runtime/storage/sqlite/data.rs", include_str!("../src/storage/sqlite/data.rs")),
    ("dokime-runtime/storage/sqlite/data/entry.rs", include_str!("../src/storage/sqlite/data/entry.rs")),
    ("dokime-runtime/storage/sqlite/data/entry/object.rs", include_str!("../src/storage/sqlite/data/entry/object.rs")),
    ("dokime-runtime/storage/sqlite/data/files.rs", include_str!("../src/storage/sqlite/data/files.rs")),
    ("dokime-runtime/storage/sqlite/data/mutations.rs", include_str!("../src/storage/sqlite/data/mutations.rs")),
    ("dokime-runtime/storage/sqlite/data/production.rs", include_str!("../src/storage/sqlite/data/production.rs")),
    ("dokime-runtime/storage/sqlite/data/records.rs", include_str!("../src/storage/sqlite/data/records.rs")),
    ("dokime-runtime/storage/sqlite/data/telemetry/catalog.rs", include_str!("../src/storage/sqlite/data/telemetry/catalog.rs")),
    ("dokime-runtime/storage/sqlite/data/telemetry/writer.rs", include_str!("../src/storage/sqlite/data/telemetry/writer.rs")),
    ("dokime-runtime/storage/sqlite/data/telemetry/aggregates.rs", include_str!("../src/storage/sqlite/data/telemetry/aggregates.rs")),
    ("dokime-runtime/storage/sqlite/data/telemetry/previews.rs", include_str!("../src/storage/sqlite/data/telemetry/previews.rs")),
    ("dokime-runtime/storage/sqlite/settings.rs", include_str!("../src/storage/sqlite/settings.rs")),
    ("dokime-runtime/storage/sqlite/history.rs", include_str!("../src/storage/sqlite/history.rs")),
    ("dokime-runtime/storage/sqlite/snapshots.rs", include_str!("../src/storage/sqlite/snapshots.rs")),
    ("dokime-runtime/plot/preview.rs", include_str!("../src/plot/preview.rs")),
    ("dokime-runtime/plot/preview/index.rs", include_str!("../src/plot/preview/index.rs")),
    ("dokime-runtime/plot/payload/types.rs", include_str!("../src/plot/payload/types.rs")),
    ("dokime-runtime/transactions/mod.rs", include_str!("../src/transactions/mod.rs")),
    ("dokime-runtime/transactions/archive.rs", include_str!("../src/transactions/archive.rs")),
    ("dokime-runtime/transactions/cycle.rs", include_str!("../src/transactions/cycle.rs")),
    ("dokime-runtime/transactions/documents.rs", include_str!("../src/transactions/documents.rs")),
    ("dokime-runtime/transactions/effects.rs", include_str!("../src/transactions/effects.rs")),
    ("dokime-runtime/transactions/evidence.rs", include_str!("../src/transactions/evidence.rs")),
    ("dokime-runtime/transactions/execution_cancel.rs", include_str!("../src/transactions/execution_cancel.rs")),
    ("dokime-runtime/transactions/execution_control.rs", include_str!("../src/transactions/execution_control.rs")),
    ("dokime-runtime/transactions/installation.rs", include_str!("../src/transactions/installation.rs")),
    ("dokime-runtime/transactions/invalidation.rs", include_str!("../src/transactions/invalidation.rs")),
    ("dokime-runtime/transactions/labels.rs", include_str!("../src/transactions/labels.rs")),
    ("dokime-runtime/transactions/lifecycle.rs", include_str!("../src/transactions/lifecycle.rs")),
    ("dokime-runtime/transactions/notes.rs", include_str!("../src/transactions/notes.rs")),
    ("dokime-runtime/transactions/outputs.rs", include_str!("../src/transactions/outputs.rs")),
    ("dokime-runtime/transactions/parameters.rs", include_str!("../src/transactions/parameters.rs")),
    ("dokime-runtime/transactions/planning.rs", include_str!("../src/transactions/planning.rs")),
    ("dokime-runtime/transactions/planning/append.rs", include_str!("../src/transactions/planning/append.rs")),
    ("dokime-runtime/transactions/planning/copy.rs", include_str!("../src/transactions/planning/copy.rs")),
    ("dokime-runtime/transactions/plugins.rs", include_str!("../src/transactions/plugins.rs")),
    ("dokime-runtime/transactions/prompts.rs", include_str!("../src/transactions/prompts.rs")),
    ("dokime-runtime/transactions/provider.rs", include_str!("../src/transactions/provider.rs")),
    ("dokime-runtime/transactions/provider/request.rs", include_str!("../src/transactions/provider/request.rs")),
    ("dokime-runtime/transactions/provider/request/completion.rs", include_str!("../src/transactions/provider/request/completion.rs")),
    ("dokime-runtime/transactions/provider_finish.rs", include_str!("../src/transactions/provider_finish.rs")),
    ("dokime-runtime/transactions/suite_sets.rs", include_str!("../src/transactions/suite_sets.rs")),
    ("dokime-runtime/transactions/traceability.rs", include_str!("../src/transactions/traceability.rs")),
];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn emit() {
    let mut digest = Sha256::new();
    let settings = harmos::config::SchemaSettings::default();
    let graphs = [
        ("application-settings", dokime_domain::model::settings::ApplicationSettings::DEFINITION),
        ("execution-history", dokime_domain::model::query::page::ExecutionSummary::DEFINITION),
        ("cycle-history", dokime_domain::model::query::page::CycleSummary::DEFINITION),
    ];
    for (name, definition) in graphs {
        let graph = Schema::from_declaration(definition, &settings).expect("stored schema graph");
        let bytes = serde_json::to_vec(&graph).expect("serialize stored schema graph");
        input(&mut digest, name, &bytes);
    }
    for (name, text) in dokime_domain::storage_format::SOURCES.iter().chain(SOURCES) {
        input(&mut digest, name, source::digest(text).as_bytes());
    }
    println!("cargo:rustc-env=DOKIME_STORAGE_FORMAT={}", source::hex(&digest.finalize()));

    // Include-Str Tracks Rust Inputs; Git Refs Must Be Tracked Separately.
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo supplies it"));
    println!("cargo:rerun-if-env-changed=DOKIME_BUILD_COMMIT");
    let revision = env::var("DOKIME_BUILD_COMMIT")
        .ok()
        .or_else(|| {
            let bytes = std::fs::read(manifest.join(".cargo_vcs_info.json")).ok()?;
            let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
            value["git"]["sha1"].as_str().map(str::to_owned)
        })
        .or_else(|| git(&manifest, &["rev-parse", "HEAD"]))
        .expect("storage writer commit requires Git, Cargo VCS metadata or DOKIME_BUILD_COMMIT");
    for reference in ["HEAD".to_owned(), "packed-refs".to_owned()].into_iter().chain(git(&manifest, &["symbolic-ref", "-q", "HEAD"])) {
        if let Some(path) = git(&manifest, &["rev-parse", "--git-path", &reference]) {
            println!("cargo:rerun-if-changed={}", manifest.join(path).display());
        }
    }
    for (name, _) in SOURCES {
        let relative = name.strip_prefix("dokime-runtime/").expect("runtime source input");
        println!("cargo:rerun-if-changed=src/{relative}");
    }
    println!("cargo:rerun-if-changed=build");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-env=DOKIME_STORAGE_BUILD={revision}");
}

/* === Private Functions === */

fn input(digest: &mut Sha256, name: &str, bytes: &[u8]) {
    digest.update((name.len() as u64).to_le_bytes());
    digest.update(name.as_bytes());
    digest.update((bytes.len() as u64).to_le_bytes());
    digest.update(bytes);
}

fn git(root: &std::path::Path, arguments: &[&str]) -> Option<String> {
    let output = dokime_domain::process::command("git").current_dir(root).args(arguments).output().ok()?;
    output.status.success().then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/* === Tests === */
