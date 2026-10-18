/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(super) mod canonical;
pub(super) mod projection;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
// Third Party Imports
use tonic::Status;
// Local Imports

/* === Constants === */
pub(super) const MAX_COLLECTION: usize = 256;
// Every text field the wire admits, labels included.
pub(super) const MAX_TEXT_BYTES: usize = 4096;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Budget(usize);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Budget {
    pub(super) fn new() -> Self {
        Self(4096)
    }
    pub(super) fn node(&mut self, depth: usize) -> Result<(), Status> {
        if depth > 32 || self.0 == 0 {
            return Err(Status::resource_exhausted("value/expression budget exceeded"));
        }
        self.0 -= 1;
        Ok(())
    }
}

/* === Public Functions === */
pub(super) fn collection(len: usize) -> Result<(), Status> {
    if len > MAX_COLLECTION {
        return Err(Status::resource_exhausted("collection exceeds 256 items"));
    }
    Ok(())
}

pub(super) fn text(value: &str) -> Result<(), Status> {
    if value.len() > MAX_TEXT_BYTES {
        return Err(Status::resource_exhausted("text exceeds 4096 bytes"));
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
