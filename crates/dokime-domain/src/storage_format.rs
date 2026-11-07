/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Stored Domain Source Inputs

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

// Only Domain Types Reached By Snapshots, Journals, Settings And Evidence.
// Embedded Here So Published Crates Do Not Need A Sibling Source Checkout.
pub const SOURCES: &[(&str, &str)] = &[
    ("dokime-domain/model/records/publication.rs", include_str!("model/records/publication.rs")),
    ("dokime-domain/model/diagnostics/capture.rs", include_str!("model/diagnostics/capture.rs")),
    ("dokime-domain/model/records/error.rs", include_str!("model/records/error.rs")),
    ("dokime-domain/model/execution/command.rs", include_str!("model/execution/command.rs")),
    ("dokime-domain/model/prompts/validation.rs", include_str!("model/prompts/validation.rs")),
    ("dokime-domain/model/validation/evaluation.rs", include_str!("model/validation/evaluation.rs")),
    ("dokime-domain/model/query/page.rs", include_str!("model/query/page.rs")),
    ("dokime-domain/model/history/wire.rs", include_str!("model/history/wire.rs")),
    ("dokime-domain/model/execution/wire.rs", include_str!("model/execution/wire.rs")),
    ("dokime-domain/model/identity/wire.rs", include_str!("model/identity/wire.rs")),
    ("dokime-domain/model/remote.rs", include_str!("model/remote.rs")),
    ("dokime-domain/model/plugins/product/registration.rs", include_str!("model/plugins/product/registration.rs")),
    ("dokime-domain/model/plugins/dispatch.rs", include_str!("model/plugins/dispatch.rs")),
    ("dokime-domain/model/settings/mod.rs", include_str!("model/settings/mod.rs")),
    ("dokime-domain/model/settings/boolean.rs", include_str!("model/settings/boolean.rs")),
    ("dokime-domain/model/settings/multiple_choice.rs", include_str!("model/settings/multiple_choice.rs")),
    ("dokime-domain/model/settings/number.rs", include_str!("model/settings/number.rs")),
    ("dokime-domain/model/settings/path.rs", include_str!("model/settings/path.rs")),
    ("dokime-domain/model/settings/secret.rs", include_str!("model/settings/secret.rs")),
    ("dokime-domain/model/settings/text_area.rs", include_str!("model/settings/text_area.rs")),
    ("dokime-domain/model/telemetry/sampling/request.rs", include_str!("model/telemetry/sampling/request.rs")),
    ("dokime-domain/model/analysis/mod.rs", include_str!("model/analysis/mod.rs")),
    ("dokime-domain/model/appearance/card.rs", include_str!("model/appearance/card.rs")),
    ("dokime-domain/model/artifacts/record.rs", include_str!("model/artifacts/record.rs")),
    ("dokime-domain/model/definition/conditions.rs", include_str!("model/definition/conditions.rs")),
    ("dokime-domain/model/definition/parameters.rs", include_str!("model/definition/parameters.rs")),
    ("dokime-domain/model/definition/suite.rs", include_str!("model/definition/suite.rs")),
    ("dokime-domain/model/definition/registry.rs", include_str!("model/definition/registry.rs")),
    ("dokime-domain/model/diagnostics/error.rs", include_str!("model/diagnostics/error.rs")),
    ("dokime-domain/model/diagnostics/debug.rs", include_str!("model/diagnostics/debug.rs")),
    ("dokime-domain/model/document/addition.rs", include_str!("model/document/addition.rs")),
    ("dokime-domain/model/document/definition.rs", include_str!("model/document/definition.rs")),
    ("dokime-domain/model/execution/effects.rs", include_str!("model/execution/effects.rs")),
    ("dokime-domain/model/execution/health.rs", include_str!("model/execution/health.rs")),
    ("dokime-domain/model/execution/mode.rs", include_str!("model/execution/mode.rs")),
    ("dokime-domain/model/execution/notes.rs", include_str!("model/execution/notes.rs")),
    ("dokime-domain/model/execution/request.rs", include_str!("model/execution/request.rs")),
    ("dokime-domain/model/execution/result.rs", include_str!("model/execution/result.rs")),
    ("dokime-domain/model/execution/saved.rs", include_str!("model/execution/saved.rs")),
    ("dokime-domain/model/execution/scope.rs", include_str!("model/execution/scope.rs")),
    ("dokime-domain/model/execution/snapshot.rs", include_str!("model/execution/snapshot.rs")),
    ("dokime-domain/model/execution/termination.rs", include_str!("model/execution/termination.rs")),
    ("dokime-domain/model/expressions/data.rs", include_str!("model/expressions/data.rs")),
    ("dokime-domain/model/expressions/expression.rs", include_str!("model/expressions/expression.rs")),
    ("dokime-domain/model/identity/id.rs", include_str!("model/identity/id.rs")),
    ("dokime-domain/model/identity/identities.rs", include_str!("model/identity/identities.rs")),
    ("dokime-domain/model/identity/pattern.rs", include_str!("model/identity/pattern.rs")),
    ("dokime-domain/model/plugins/appearance.rs", include_str!("model/plugins/appearance.rs")),
    ("dokime-domain/model/plugins/contribution/binding.rs", include_str!("model/plugins/contribution/binding.rs")),
    ("dokime-domain/model/plugins/contribution/customization.rs", include_str!("model/plugins/contribution/customization.rs")),
    ("dokime-domain/model/plugins/contribution/options.rs", include_str!("model/plugins/contribution/options.rs")),
    ("dokime-domain/model/plugins/error.rs", include_str!("model/plugins/error.rs")),
    ("dokime-domain/model/plugins/installed.rs", include_str!("model/plugins/installed.rs")),
    ("dokime-domain/model/plugins/permission.rs", include_str!("model/plugins/permission.rs")),
    ("dokime-domain/model/plugins/release.rs", include_str!("model/plugins/release.rs")),
    ("dokime-domain/model/prompts/mod.rs", include_str!("model/prompts/mod.rs")),
    ("dokime-domain/model/prompts/rules.rs", include_str!("model/prompts/rules.rs")),
    ("dokime-domain/model/records/contract/conditions.rs", include_str!("model/records/contract/conditions.rs")),
    ("dokime-domain/model/records/contract/pattern.rs", include_str!("model/records/contract/pattern.rs")),
    ("dokime-domain/model/records/contract/schema.rs", include_str!("model/records/contract/schema.rs")),
    ("dokime-domain/model/records/contract/severity.rs", include_str!("model/records/contract/severity.rs")),
    ("dokime-domain/model/records/evidence/entry.rs", include_str!("model/records/evidence/entry.rs")),
    ("dokime-domain/model/records/evidence/marker.rs", include_str!("model/records/evidence/marker.rs")),
    ("dokime-domain/model/records/evidence/record.rs", include_str!("model/records/evidence/record.rs")),
    ("dokime-domain/model/records/resources.rs", include_str!("model/records/resources.rs")),
    ("dokime-domain/model/settings/application.rs", include_str!("model/settings/application.rs")),
    ("dokime-domain/model/secrets/catalog.rs", include_str!("model/secrets/catalog.rs")),
    ("dokime-domain/model/telemetry/contract/conditions.rs", include_str!("model/telemetry/contract/conditions.rs")),
    ("dokime-domain/model/telemetry/contract/filter.rs", include_str!("model/telemetry/contract/filter.rs")),
    ("dokime-domain/model/telemetry/contract/schema.rs", include_str!("model/telemetry/contract/schema.rs")),
    ("dokime-domain/model/telemetry/evidence/batch.rs", include_str!("model/telemetry/evidence/batch.rs")),
    ("dokime-domain/model/telemetry/evidence/record.rs", include_str!("model/telemetry/evidence/record.rs")),
    ("dokime-domain/model/telemetry/sampling/source.rs", include_str!("model/telemetry/sampling/source.rs")),
    ("dokime-domain/model/time/clock.rs", include_str!("model/time/clock.rs")),
    ("dokime-domain/model/traceability/mod.rs", include_str!("model/traceability/mod.rs")),
    ("dokime-domain/model/values/format.rs", include_str!("model/values/format.rs")),
    ("dokime-domain/model/values/unit.rs", include_str!("model/values/unit.rs")),
    ("dokime-domain/model/values/value.rs", include_str!("model/values/value.rs")),
    ("dokime-domain/state/records/mod.rs", include_str!("state/records/mod.rs")),
    ("dokime-domain/state/records/resources.rs", include_str!("state/records/resources.rs")),
    ("dokime-domain/state/records/set.rs", include_str!("state/records/set.rs")),
    ("dokime-domain/state/records/work.rs", include_str!("state/records/work.rs")),
];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
