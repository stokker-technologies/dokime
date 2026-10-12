/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod appearance;
pub(crate) mod context;
pub(crate) mod records;
pub(crate) mod source;
pub(crate) mod telemetry;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports

// Local Imports
use crate::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */
pub(crate) trait Encode: harmos::Protobuf {
    fn encoded_length(&self, maximum: NonZeroUsize) -> Result<usize> {
        let length = self.encoded_len();
        if length > maximum.get() {
            return Err(invalid("native captured report exceeds file byte limit"));
        }
        Ok(length)
    }

    fn encode_counted(&self, length: usize) -> Result<Vec<u8>> {
        if self.encoded_len() != length {
            return Err(invalid(
                "native captured report length differs from preclaim",
            ));
        }
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(length)
            .map_err(|_| invalid("native captured report output allocation failed"))?;
        if bytes.capacity() > length {
            return Err(invalid("native captured report output exceeds admission"));
        }
        self.encode_into(&mut bytes);
        if bytes.len() != length || bytes.capacity() > length {
            return Err(invalid(
                "native captured report output differs from preclaim",
            ));
        }
        Ok(bytes)
    }
}

/* === Implementations === */
impl<T: harmos::Protobuf> Encode for T {}

/* === Public Functions === */

/* === Private Functions === */
fn invalid(message: &str) -> Error {
    Error::InvalidReportInputs(message.into())
}

/* === Tests === */
