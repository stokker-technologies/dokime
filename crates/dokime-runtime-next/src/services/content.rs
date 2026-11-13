/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Content Resource Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl dokime_protocol::client::content_server::Content for Services {
    type DownloadAppearanceAssetStream =
        dokime_operations_next::client::Output<dokime_protocol::client::ContentDownloadAppearanceAssetOutput>;
    async fn download_appearance_asset(
        &self, request: tonic::Request<dokime_protocol::client::ContentDownloadAppearanceAssetRequest>,
    ) -> Result<tonic::Response<Self::DownloadAppearanceAssetStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("dokime.client.v1.Content/DownloadAppearanceAsset: Operation Is Unavailable in This Host"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
