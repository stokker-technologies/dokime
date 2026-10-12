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
use std::{
    task::{Context, Poll},
    time::Duration,
};
// Third Party Imports
use futures_util::future::BoxFuture;
use tonic::codegen::http::{Request, Response};
use tower::{Layer, Service};
// Local Imports
use super::method::{self, Work};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone)]
pub(super) struct Deadlines {
    pub ordinary: Duration,
    pub upload: Duration,
}

#[derive(Clone)]
pub(super) struct DeadlineService<S> {
    inner: S,
    deadlines: Deadlines,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Default for Deadlines {
    fn default() -> Self {
        Self {
            ordinary: Duration::from_secs(10),
            upload: Duration::from_secs(300),
        }
    }
}

impl<S> Layer<S> for Deadlines {
    type Service = DeadlineService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        DeadlineService {
            inner,
            deadlines: self.clone(),
        }
    }
}

impl<S, B> Service<Request<B>> for DeadlineService<S>
where
    S: Service<Request<B>, Response = Response<tonic::body::Body>>,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }
    fn call(&mut self, request: Request<B>) -> Self::Future {
        // The method table owns the classification. Report generation walks every
        // checkpoint in a cycle (rendering HTML, plots and traceability) and byte
        // transfer moves whole artifacts, so both legitimately outrun the ordinary
        // deadline; a metadata read never does.
        let duration = match method::find(request.uri().path()).map(|method| method.work) {
            Some(Work::Transfer) => self.deadlines.upload,
            _ => self.deadlines.ordinary,
        };
        let future = self.inner.call(request);
        Box::pin(async move {
            match tokio::time::timeout(duration, future).await {
                Ok(result) => result,
                Err(_) => Ok(
                    tonic::Status::deadline_exceeded("backend request deadline exceeded")
                        .into_http(),
                ),
            }
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
