/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(super) mod canonical;
pub(super) mod projection;

use tonic::Status;

const SERVER_MAXIMUM_COLLECTION_ITEMS: u64 = 256;
const SERVER_VALUE_DEPTH: u64 = 32;
const SERVER_VALUE_NODES: u64 = 4_096;

pub(super) struct Budget {
    remaining: usize,
    maximum_depth: usize,
}

impl Budget {
    pub(super) fn new() -> Result<Self, Status> {
        let remaining = usize::try_from(SERVER_VALUE_NODES).map_err(|_| Status::invalid_argument("value nodes exceed native width"))?;
        let maximum_depth =
            usize::try_from(SERVER_VALUE_DEPTH).map_err(|_| Status::invalid_argument("value depth exceeds native width"))?;
        Ok(Self { remaining, maximum_depth })
    }
    pub(super) fn node(&mut self, depth: usize) -> Result<(), Status> {
        if depth > self.maximum_depth || self.remaining == 0 {
            return Err(Status::resource_exhausted("value/expression budget exceeded"));
        }
        self.remaining -= 1;
        Ok(())
    }
}

pub(super) fn collection(len: usize) -> Result<(), Status> {
    if len as u64 > SERVER_MAXIMUM_COLLECTION_ITEMS {
        return Err(Status::resource_exhausted("collection exceeds its configured expansion limit"));
    }
    Ok(())
}
