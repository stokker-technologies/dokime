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
use super::{ApplicationSettings, ORDINARY, ReadApplicationSettingsRequest};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// The backend's settings.json: appearance and the data root. Saved values apply to the next start of the backend; what is in effect now is reported beside them.
#[harmos::operation(request = ReadApplicationSettingsRequest, response = ApplicationSettings, id = "dokime.settings.read_application_settings", version = 1,
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Settings", method = "ReadApplicationSettings"))]
pub struct ReadApplicationSettings;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
