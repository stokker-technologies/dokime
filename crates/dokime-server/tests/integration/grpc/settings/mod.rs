/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Settings, appearance assets and authorization boundaries over the served surface.

/* === Modules === */

mod appearance;
mod authorization;
mod persistence;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[dokime_tracing::trace_test]
#[tokio::test]
async fn settings() {
    stage(Box::pin(persistence::frame_readable_probe())).await;
    stage(Box::pin(persistence::storage_sources())).await;
    stage(Box::pin(persistence::plugin_settings())).await;
    stage(Box::pin(persistence::application_settings())).await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn appearance() {
    stage(Box::pin(appearance::appearance_asset_bound())).await;
    stage(Box::pin(appearance::appearance_refuses_transfer())).await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn authorization() {
    authorization::table_covers_serves();
    stage(Box::pin(authorization::key_boundary_matrix())).await;
    stage(Box::pin(authorization::remote_refuses_local())).await;
}
