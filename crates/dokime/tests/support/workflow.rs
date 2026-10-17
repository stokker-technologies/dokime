/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The one pause between drains of a live execution. `Workflow::drain` is a
// cursor read with no public wait on the execution's change notification, so
// every drain loop in these stories pumps through this single site.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports

/* === Constants === */
const POLL: Duration = Duration::from_millis(2);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
// TODO(source-hook): `Workflow::changed(token)`, a public wait on the execution's `changed` notification.
pub async fn next_change() {
    tokio::time::sleep(POLL).await;
}

/* === Private Functions === */

/* === Tests === */
