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
use tonic::Status;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn response_limit(value: &impl harmos::Protobuf, maximum: usize) -> Result<(), Status> {
    if harmos::Protobuf::encoded_len(value) > maximum {
        return Err(Status::resource_exhausted("receipt exceeds response budget"));
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
