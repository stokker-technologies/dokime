/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::Deserialize;
use serde::Serialize;

use super::*;

// At most 8 MiB per registered appearance asset. Capture and verify before
// streaming: one metadata frame, contiguous <=64 KiB chunks, completion and OK
// final status. A changed/missing reference requires refreshing GetAppearance.
#[harmos::operation(resources = [crate::hosting::Host<DownloadAppearanceAsset>], response = (), output = AppearanceAssetFrame, id = "dokime.content.download_appearance_asset",
        policy = TRANSFER, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Content", method = "DownloadAppearanceAsset"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadAppearanceAsset {
    pub reference: Option<AppearanceAssetReference>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(DownloadAppearanceAsset);
}
