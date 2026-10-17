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
use harmos::MessageError;
use harmos::domain::Portable;
use harmos::grpc::{Transcoded, Wire};
use tonic::Status;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug, thiserror::Error)]
#[error("message encoding failed: {0}")]
pub(super) struct Untranscodable(#[from] MessageError);

/* === Enums === */

/* === Traits === */
pub(super) trait Transcode {
    fn wire_bytes(&self) -> Result<Vec<u8>, Untranscodable>;

    fn wire_len(&self) -> Result<usize, Untranscodable> {
        self.wire_bytes().map(|bytes| bytes.len())
    }
}

/* === Implementations === */
impl<T: Portable> Transcode for T {
    fn wire_bytes(&self) -> Result<Vec<u8>, Untranscodable> {
        let mut bytes = Vec::new();
        <Transcoded as Wire<T>>::encode(self, &mut bytes)?;
        Ok(bytes)
    }
}

impl From<Untranscodable> for Status {
    fn from(error: Untranscodable) -> Self {
        Status::internal(error.to_string())
    }
}

impl From<Untranscodable> for harmos::Error {
    fn from(error: Untranscodable) -> Self {
        harmos::Error::Backend { operation: "encode gRPC message", kind: harmos::BackendFailure::Permanent, source: Box::new(error) }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
