/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Organization Pack Lifecycle Stories

/* === Modules === */

use dokime_gui::testing as implementation;

mod installed_appearance;
mod malformed_policy_diagnostics_do_not_echo_manifest_values;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::config::organization::resolve_directory;

/* === Constants === */

const MANIFEST: &str = "organization.toml";

const LOGO: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg'/>";
const MANIFEST_TEXT: &str = "[app]\nlabel = \"Stokker Technologies\"\n[app.logos.transparent]\non_light = \"logos/mark.svg\"\n";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
