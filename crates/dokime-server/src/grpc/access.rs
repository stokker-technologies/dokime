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
pub use dokime::operations::Access as Role;

/* === Imports === */
// Standard Library Imports
use std::{
    sync::Arc,
    task::{Context, Poll},
};
// Third Party Imports
use futures_util::future::BoxFuture;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use tonic::codegen::http::{self, HeaderMap, Request as HttpRequest, Response as HttpResponse};
use tonic::{Request, Status};
use tower::{Layer, Service};
// Local Imports
use super::{Error, method};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Principal {
    subject: String,
    role: Role,
}

#[derive(Clone)]
pub struct BearerGrant {
    principal: Principal,
    digest: [u8; 32],
}

#[derive(Clone, Debug)]
pub(crate) struct Authorize {
    policy: Policy,
}

#[derive(Clone)]
pub(crate) struct AuthorizeService<S> {
    inner: S,
    authorize: Authorize,
}

/* === Enums === */
#[derive(Clone, Debug)]
pub(crate) enum Policy {
    TrustedLocal,
    Bearer(Arc<[BearerGrant]>),
}

/* === Traits === */

/* === Implementations === */
impl Principal {
    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn role(&self) -> Role {
        self.role
    }

    pub fn of<T>(request: &Request<T>) -> Result<&Self, Status> {
        request
            .extensions()
            .get::<Self>()
            .ok_or_else(|| Status::unauthenticated("authentication required"))
    }
}

impl std::fmt::Debug for BearerGrant {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BearerGrant")
            .finish_non_exhaustive()
    }
}

impl BearerGrant {
    pub fn new(subject: String, role: Role, token: &str) -> Result<Self, Error> {
        if subject.trim().is_empty() || subject.len() > 256 || subject.chars().any(char::is_control)
        {
            return Err(Error::Configuration("invalid principal subject"));
        }
        if !(32..=4096).contains(&token.len()) || !token.bytes().all(|byte| byte.is_ascii_graphic())
        {
            return Err(Error::Configuration(
                "bearer tokens require 32–4096 visible ASCII bytes",
            ));
        }
        Ok(Self {
            principal: Principal { subject, role },
            digest: Sha256::digest(token.as_bytes()).into(),
        })
    }
}

impl Policy {
    pub(crate) fn bearer(grants: Vec<BearerGrant>) -> Result<Self, Error> {
        if grants.is_empty() {
            return Err(Error::Configuration(
                "bearer authentication requires at least one grant",
            ));
        }
        for (index, grant) in grants.iter().enumerate() {
            if grants[..index]
                .iter()
                .any(|other| bool::from(grant.digest.ct_eq(&other.digest)))
            {
                return Err(Error::Configuration("duplicate bearer token grants"));
            }
        }
        Ok(Self::Bearer(grants.into()))
    }

    fn authenticate(&self, headers: &HeaderMap) -> Result<Principal, Status> {
        let Self::Bearer(grants) = self else {
            return Ok(Principal {
                subject: "local".into(),
                role: Role::Admin,
            });
        };
        let mut credentials = headers.get_all(http::header::AUTHORIZATION).into_iter();
        let value = credentials
            .next()
            .ok_or_else(|| Status::unauthenticated("authentication required"))?;
        if credentials.next().is_some() {
            return Err(Status::unauthenticated("invalid credentials"));
        }
        let token = value
            .to_str()
            .ok()
            .and_then(|value| {
                let (scheme, token) = value.split_once(' ')?;
                (scheme.eq_ignore_ascii_case("bearer")
                    && (32..=4096).contains(&token.len())
                    && token.bytes().all(|byte| byte.is_ascii_graphic()))
                .then_some(token)
            })
            .ok_or_else(|| Status::unauthenticated("invalid credentials"))?;
        let digest: [u8; 32] = Sha256::digest(token.as_bytes()).into();
        // Compare all fixed-size digests; never early-return on a matching grant.
        let mut matched = None;
        for (index, grant) in grants.iter().enumerate() {
            if bool::from(digest.ct_eq(&grant.digest)) {
                matched = Some(index);
            }
        }
        matched
            .map(|index| grants[index].principal.clone())
            .ok_or_else(|| Status::unauthenticated("invalid credentials"))
    }
}

impl Authorize {
    pub(crate) fn new(policy: Policy) -> Self {
        Self { policy }
    }

    fn admit<B>(&self, request: &mut HttpRequest<B>) -> Result<(), Status> {
        let principal = self.policy.authenticate(request.headers())?;
        let method = method::find(request.uri().path())
            .ok_or_else(|| Status::unimplemented("method is not served by this listener"))?;
        if principal.role < method.role {
            return Err(Status::permission_denied("insufficient permission"));
        }
        // Credentials must not propagate to service logging or application code.
        request.headers_mut().remove(http::header::AUTHORIZATION);
        request.extensions_mut().insert(principal);
        Ok(())
    }
}

impl<S> Layer<S> for Authorize {
    type Service = AuthorizeService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthorizeService {
            inner,
            authorize: self.clone(),
        }
    }
}

impl<S, B> Service<HttpRequest<B>> for AuthorizeService<S>
where
    S: Service<HttpRequest<B>, Response = HttpResponse<tonic::body::Body>>,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: HttpRequest<B>) -> Self::Future {
        if let Err(status) = self.authorize.admit(&mut request) {
            return Box::pin(async move { Ok(status.into_http()) });
        }
        let future = self.inner.call(request);
        Box::pin(future)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
