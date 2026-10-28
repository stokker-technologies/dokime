/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use tonic::Status;

pub(super) fn response_limit(value: &impl dokime::harmos::domain::Portable, maximum: usize) -> Result<(), Status> {
    if crate::grpc::transcode::Transcode::wire_len(value)? > maximum {
        return Err(Status::resource_exhausted("receipt exceeds response budget"));
    }
    Ok(())
}
