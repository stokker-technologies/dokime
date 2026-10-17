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
use std::marker::PhantomData;
use std::sync::Arc;
// Third Party Imports

// Local Imports
use dokime_domain::model::identity::Identifier;
use crate::telemetry::{Streams, TelemetryStream};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct LeaseHold {
    pub lease_token: String,
    pub release: Arc<dyn LeaseRelease>,
}

pub struct Lease<'a, R> {
    stream: Identifier,
    lease_token: String,
    release: Arc<dyn LeaseRelease>,
    streams: Streams,
    row: PhantomData<fn() -> R>,
    borrowed: PhantomData<&'a ()>,
}

/* === Enums === */

/* === Traits === */
pub trait LeaseRelease: Send + Sync {
    fn release(&self, lease_token: &str);
}

/* === Implementations === */
impl<'a, R> Lease<'a, R> {
    pub fn new(stream: Identifier, hold: LeaseHold, streams: Streams) -> Self {
        Self {
            stream,
            lease_token: hold.lease_token,
            release: hold.release,
            streams,
            row: PhantomData,
            borrowed: PhantomData,
        }
    }

    pub fn stream(&self) -> &Identifier {
        &self.stream
    }
}

impl<R: TelemetryStream> Lease<'_, R> {
    #[cfg(feature = "native")]
    pub fn listen(&self) -> crate::execution::listen::ListeningRows<R> {
        crate::execution::listen::ListeningRows::new(&self.streams)
    }
}

impl<R> Drop for Lease<'_, R> {
    fn drop(&mut self) {
        self.release.release(&self.lease_token);
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
