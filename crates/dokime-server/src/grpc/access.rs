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
use std::{collections::BTreeSet, sync::Arc};
// Third Party Imports
use harmos::plugin::Grant;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use tonic::{Request, Status};
// Local Imports
use super::Error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationPermission {
    pub owner: String,
    pub id: String,
    #[serde(default)]
    pub grants: Vec<Grant>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Principal {
    subject: String,
    role: Role,
    operations: Arc<[OperationPermission]>,
}

#[derive(Clone)]
pub struct BearerGrant {
    principal: Principal,
    digest: [u8; 32],
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

    pub(crate) fn operation(&self, owner: &str, id: &str) -> Option<&OperationPermission> {
        self.operations
            .iter()
            .find(|permission| permission.owner == owner && permission.id == id)
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
            principal: Principal {
                subject,
                role,
                operations: Arc::from([]),
            },
            digest: Sha256::digest(token.as_bytes()).into(),
        })
    }
}

impl BearerGrant {
    pub fn with_operations(mut self, operations: Vec<OperationPermission>) -> Result<Self, Error> {
        if operations.len() > 128 {
            return Err(Error::Configuration(
                "at most 128 exact plugin operation permissions are allowed",
            ));
        }
        let mut seen = BTreeSet::new();
        for permission in &operations {
            harmos::Identifier::new(&permission.owner)
                .map_err(|_| Error::Configuration("invalid plugin permission owner"))?;
            harmos::Identifier::new(&permission.id)
                .map_err(|_| Error::Configuration("invalid plugin permission operation"))?;
            if permission.owner == "dokime" || permission.owner == "harmos" {
                return Err(Error::Configuration(
                    "plugin permissions cannot replace product or host policy",
                ));
            }
            if !seen.insert((&permission.owner, &permission.id)) {
                return Err(Error::Configuration(
                    "duplicate plugin operation permission",
                ));
            }
            if permission.grants.len() > 32 {
                return Err(Error::Configuration(
                    "at most 32 scoped grants are allowed per operation",
                ));
            }
            let mut grants = BTreeSet::new();
            for grant in &permission.grants {
                let exact = grant.operation() == permission.id
                    && grant.scope().is_some_and(|scope| {
                        !scope.trim().is_empty()
                            && scope.len() <= 1024
                            && !scope
                                .chars()
                                .any(|character| character.is_control() || character == '*')
                    });
                if !exact || !grants.insert(grant) {
                    return Err(Error::Configuration(
                        "scoped grants require distinct bounded exact scopes on the permitted operation",
                    ));
                }
            }
        }
        self.principal.operations = operations.into();
        Ok(self)
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

    fn authenticate_values<'a>(
        &self,
        mut credentials: impl Iterator<Item = Option<&'a str>>,
    ) -> Result<Principal, Status> {
        let Self::Bearer(grants) = self else {
            return Ok(Principal {
                subject: "local".into(),
                role: Role::Admin,
                operations: Arc::from([]),
            });
        };
        let value = credentials
            .next()
            .ok_or_else(|| Status::unauthenticated("authentication required"))?
            .ok_or_else(|| Status::unauthenticated("invalid credentials"))?;
        if credentials.next().is_some() {
            return Err(Status::unauthenticated("invalid credentials"));
        }
        let (_, token) = value
            .split_once(' ')
            .filter(|(scheme, token)| {
                scheme.eq_ignore_ascii_case("bearer")
                    && (32..=4096).contains(&token.len())
                    && token.bytes().all(|byte| byte.is_ascii_graphic())
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

impl harmos::grpc::authentication::Authenticator for Policy {
    type Principal = Principal;

    fn authenticate<'a>(
        &'a self,
        call: &'a harmos::grpc::middleware::Invocation,
    ) -> harmos::grpc::authentication::Authentication<'a, Self::Principal> {
        Box::pin(async move {
            self.authenticate_values(
                call.metadata()
                    .get_all("authorization")
                    .iter()
                    .map(|value| value.to_str().ok()),
            )
            // Carry the authored status so the served error keeps its message.
            .map_err(|status| harmos::Error::OperationFailed {
                operation: call.path().to_owned(),
                source: Arc::new(status),
            })
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
