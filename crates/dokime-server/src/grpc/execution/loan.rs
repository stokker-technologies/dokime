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
use std::num::NonZeroUsize;

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
pub(super) fn reserve(
    runtime: &dokime_app::RuntimeHandle,
    bytes: usize,
) -> Result<harmos::storage::StorageProductionPreparation, Status> {
    let bytes = NonZeroUsize::new(bytes)
        .ok_or_else(|| Status::resource_exhausted("invalid receipt memory allowance"))?;
    runtime
        .storage()
        .prepare_production(bytes)
        .map_err(memory_error)
}

pub(super) fn memory_error(error: harmos::Error) -> Status {
    let error = dokime_app::Error::from(error);
    super::super::error::detailed(Status::resource_exhausted(error.to_string()), &error)
}

pub(super) fn response_limit(value: &impl harmos::Protobuf, maximum: usize) -> Result<(), Status> {
    if harmos::Protobuf::encoded_len(value) > maximum {
        return Err(Status::resource_exhausted(
            "receipt exceeds response budget",
        ));
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
