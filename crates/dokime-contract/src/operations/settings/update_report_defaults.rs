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
use super::{ApplicationSettings, ORDINARY, UpdateReportDefaultsRequest};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::operation(request = UpdateReportDefaultsRequest, response = ApplicationSettings, id = "dokime.settings.update_report_defaults", version = 1,
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Settings", method = "UpdateReportDefaults"))]
pub struct UpdateReportDefaults;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
