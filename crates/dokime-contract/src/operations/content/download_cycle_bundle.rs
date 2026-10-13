/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports
use super::{CycleBundleFrame, DownloadCycleBundleRequest, TRANSFER};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Reserves cycle bundle export. The native runtime currently refuses it as unsupported.
#[harmos::operation(request = DownloadCycleBundleRequest, response = (), output = CycleBundleFrame, id = "dokime.content.download_cycle_bundle", version = 1,
        policy = TRANSFER, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Content", method = "DownloadCycleBundle"))]
pub struct DownloadCycleBundle;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
