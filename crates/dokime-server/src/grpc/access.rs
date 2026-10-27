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
    collections::BTreeMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};

// Third Party Imports
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use tonic::{Request, Status};

// Local Imports
use super::Error;
use super::remote::sessions::{Outcome, SessionLog};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Principal {
    subject: &'static str,
    local: bool,
}

pub(super) struct Listeners {
    policies: RwLock<BTreeMap<SocketAddr, (Policy, bool)>>,
    sessions: Arc<SessionLog>,
}

/* === Enums === */

#[derive(Clone, Debug)]
pub(crate) enum Policy {
    TrustedLocal,
    Open,
    Key([u8; 32]),
}

/* === Traits === */

/* === Implementations === */

impl Principal {
    pub(crate) fn is_local(&self) -> bool {
        self.local
    }

    pub fn subject(&self) -> &str {
        self.subject
    }

    pub fn of<T>(request: &Request<T>) -> Result<&Self, Status> {
        request.extensions().get::<Self>().ok_or_else(|| Status::unauthenticated("authentication required"))
    }
}

impl Policy {
    /// The recorded principal of an admitted remote call.
    fn principal(&self) -> &'static str {
        if matches!(self, Self::Key(_)) { "access-key" } else { "anonymous" }
    }

    fn authenticate_values<'a>(&self, mut secrets: impl Iterator<Item = Option<&'a str>>) -> Result<Principal, Status> {
        if matches!(self, Self::TrustedLocal | Self::Open) {
            let subject = if matches!(self, Self::TrustedLocal) { "local" } else { "remote" };
            return Ok(Principal { subject, local: false });
        }
        let value = secrets
            .next()
            .ok_or_else(|| Status::unauthenticated("authentication required"))?
            .ok_or_else(|| Status::unauthenticated("invalid secrets"))?;
        if secrets.next().is_some() {
            return Err(Status::unauthenticated("invalid secrets"));
        }
        let (_, token) = value
            .split_once(' ')
            .filter(|(scheme, token)| scheme.eq_ignore_ascii_case("bearer") && valid_key(token))
            .ok_or_else(|| Status::unauthenticated("invalid secrets"))?;
        let digest: [u8; 32] = Sha256::digest(token.as_bytes()).into();
        let Self::Key(expected) = self else {
            return Err(Status::unauthenticated("invalid listener policy"));
        };
        bool::from(digest.ct_eq(expected))
            .then_some(Principal { subject: "remote", local: false })
            .ok_or_else(|| Status::unauthenticated("invalid secrets"))
    }
}

impl Listeners {
    pub(super) fn new() -> Self {
        Self { policies: RwLock::new(BTreeMap::new()), sessions: Arc::new(SessionLog::new()) }
    }

    pub(super) fn insert(&self, address: SocketAddr, policy: Policy, local: bool) {
        self.policies.write().unwrap_or_else(std::sync::PoisonError::into_inner).insert(address, (policy, local));
    }

    pub(super) fn remove(&self, address: SocketAddr) {
        self.policies.write().unwrap_or_else(std::sync::PoisonError::into_inner).remove(&address);
    }

    fn policy(&self, address: SocketAddr) -> Result<(Policy, bool), Status> {
        let policies = self.policies.read().map_err(|_| Status::unavailable("listener policy unavailable"))?;
        let exact = policies.get(&address);
        let wildcard = || {
            policies.iter().find_map(|(bound, policy)| {
                (bound.port() == address.port() && bound.ip().is_unspecified() && bound.is_ipv4() == address.is_ipv4()).then_some(policy)
            })
        };
        exact.or_else(wildcard).cloned().ok_or_else(|| Status::unauthenticated("listener is not accepting calls"))
    }

    fn authenticate_call(&self, call: &dokime::harmos::grpc::middleware::Invocation) -> Result<Principal, Status> {
        use tonic::transport::server::{TcpConnectInfo, TlsConnectInfo};
        let extensions = call.transport();
        let plain = extensions.get::<TcpConnectInfo>().and_then(|info| info.local_addr);
        let tls = extensions.get::<TlsConnectInfo<TcpConnectInfo>>().and_then(|info| info.get_ref().local_addr);
        let address = tls.or(plain).ok_or_else(|| Status::unauthenticated("listener identity unavailable"))?;
        let (policy, local) = self.policy(address)?;
        let values = call.metadata().get_all("authorization");
        let authenticated = policy.authenticate_values(values.iter().map(|value| value.to_str().ok()));
        // Native Callers Are Not Remote Sessions; Remote Refusals Are Recorded With Their Reason.
        if let Some(peer) = call.peer().filter(|_| !local) {
            match &authenticated {
                Ok(_) => self.sessions.record(peer, policy.principal(), Outcome::Admitted),
                Err(status) => self.sessions.record(peer, "anonymous", Outcome::Rejected(status.message())),
            }
        }
        let mut principal = authenticated?;
        principal.local = local;
        Ok(principal)
    }

    pub(super) fn sessions(&self) -> Arc<SessionLog> {
        self.sessions.clone()
    }
}

impl dokime::harmos::grpc::authentication::Authenticator for Listeners {
    type Principal = Principal;

    fn authenticate<'a>(
        &'a self, call: &'a dokime::harmos::grpc::middleware::Invocation,
    ) -> dokime::harmos::grpc::authentication::Authentication<'a, Self::Principal> {
        Box::pin(async move {
            self.authenticate_call(call)
                // Carry the authored status so the served error keeps its message.
                .map_err(|status| dokime::harmos::Error::OperationFailed { operation: call.path().to_owned(), source: Arc::new(status) })
        })
    }
}

/* === Public Functions === */

pub(super) fn verifier(key: &str) -> Result<[u8; 32], Error> {
    if !valid_key(key) {
        return Err(Error::Configuration("access keys require 32–4096 visible ASCII bytes"));
    }
    Ok(Sha256::digest(key.as_bytes()).into())
}

/* === Private Functions === */

fn valid_key(key: &str) -> bool {
    (32..=4096).contains(&key.len()) && key.bytes().all(|byte| byte.is_ascii_graphic())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn socket_policy_isolation() {
        let listeners = Listeners::new();
        let native: SocketAddr = "127.0.0.1:45670".parse().unwrap();
        let remote: SocketAddr = "0.0.0.0:45671".parse().unwrap();
        let key = "a".repeat(64);
        let digest = Sha256::digest(key.as_bytes()).into();
        listeners.insert(native, Policy::TrustedLocal, true);
        listeners.insert(remote, Policy::Key(digest), false);
        let (native_policy, local) = listeners.policy(native).unwrap();
        assert!(local);
        assert!(native_policy.authenticate_values(std::iter::empty()).is_ok());
        let (remote_policy, local) = listeners.policy("127.0.0.1:45671".parse().unwrap()).unwrap();
        assert!(!local, "a loopback peer cannot turn remote transport into native trust");
        assert!(remote_policy.authenticate_values(std::iter::empty()).is_err());
        let secret = format!("Bearer {key}");
        let principal = remote_policy.authenticate_values([Some(secret.as_str())].into_iter()).unwrap();
        assert_eq!(principal.subject(), "remote");
        assert!(remote_policy.authenticate_values([Some(secret.as_str()), Some(secret.as_str())].into_iter()).is_err());
        assert!(remote_policy.authenticate_values([Some("Bearer wrong-key-012345678901234567890123456789")].into_iter()).is_err());
        assert!(listeners.policy("127.0.0.1:45672".parse().unwrap()).is_err());
        assert!(listeners.policy("[::1]:45671".parse().unwrap()).is_err());
        listeners.remove(remote);
        assert!(listeners.policy("127.0.0.1:45671".parse().unwrap()).is_err());
        assert!(listeners.policy(native).is_ok());
    }
}
