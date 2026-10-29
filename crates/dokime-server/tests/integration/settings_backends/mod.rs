/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Settings, appearance assets and authorization boundaries over the served surface.

mod appearance;
mod authorization;
mod settings;

use super::*;

#[tokio::test]
async fn settings_backends() {
    Box::pin(settings::frame_readable_probe()).await;
    Box::pin(settings::storage_sources()).await;
    Box::pin(settings::plugin_settings()).await;
    Box::pin(settings::application_settings()).await;
    Box::pin(appearance::appearance_asset_bound()).await;
    Box::pin(appearance::appearance_refuses_transfer()).await;
    authorization::table_covers_serves();
    Box::pin(authorization::key_boundary_matrix()).await;
    Box::pin(authorization::remote_refuses_local()).await;
}
