/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use dokime::harmos::MessageError;
use dokime::harmos::domain::Portable;
use dokime::harmos::grpc::{Transcoded, Wire};
use tonic::Status;

#[derive(Debug, thiserror::Error)]
#[error("message encoding failed: {0}")]
pub(super) struct Untranscodable(#[from] MessageError);

pub(super) trait Transcode {
    fn wire_bytes(&self) -> Result<Vec<u8>, Untranscodable>;

    fn wire_len(&self) -> Result<usize, Untranscodable>;
}

impl<T: Portable> Transcode for T {
    fn wire_bytes(&self) -> Result<Vec<u8>, Untranscodable> {
        let mut bytes = Vec::new();
        <Transcoded as Wire<T>>::encode(self, &mut bytes)?;
        Ok(bytes)
    }

    fn wire_len(&self) -> Result<usize, Untranscodable> {
        dokime::harmos::grpc::encoded_len(self).map_err(Untranscodable::from)
    }
}

impl From<Untranscodable> for Status {
    fn from(error: Untranscodable) -> Self {
        Status::internal(error.to_string())
    }
}

impl From<Untranscodable> for dokime::harmos::Error {
    fn from(error: Untranscodable) -> Self {
        dokime::harmos::Error::Backend {
            operation: "encode gRPC message",
            kind: dokime::harmos::BackendFailure::Permanent,
            source: Box::new(error),
        }
    }
}
