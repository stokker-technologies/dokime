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
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use crate::model::identity::ActorId;
use crate::model::identity::NodeId;
use crate::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DokimeIdentity {
    pub node: DokimeNodeIdentity,
    pub actor: Option<DokimeActorIdentity>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DokimeNodeIdentity {
    pub node_id: NodeId,
    pub hostname_label: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DokimeActorIdentity {
    pub actor_id: ActorId,
    pub display_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalIdentityMapping {
    pub actor_id: ActorId,
    pub provider: ExternalIdentityProvider,
    pub provider_subject: String,
    pub tenant: Option<String>,
    pub display_name: Option<String>,
}

/* === Enums === */

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "provider")]
pub enum ExternalIdentityProvider {
    MicrosoftEntra,
    Atlassian,
    Other { provider_id: String },
}

/* === Traits === */

/* === Implementations === */

impl DokimeIdentity {
    pub fn node_only(node: DokimeNodeIdentity) -> Self {
        Self { node, actor: None }
    }
}

impl DokimeNodeIdentity {
    pub fn new(node_id: NodeId, hostname_label: Option<String>, timestamp: Timestamp) -> Self {
        Self {
            node_id,
            hostname_label,
            created_at: timestamp.clone(),
            updated_at: timestamp,
        }
    }
}

impl DokimeActorIdentity {
    pub fn local(actor_id: ActorId, display_name: impl Into<String>) -> Self {
        Self {
            actor_id,
            display_name: display_name.into(),
        }
    }
}

impl ExternalIdentityMapping {
    pub fn microsoft_entra(
        actor_id: ActorId, tenant: impl Into<String>, provider_subject: impl Into<String>, display_name: Option<String>,
    ) -> Self {
        Self {
            actor_id,
            provider: ExternalIdentityProvider::MicrosoftEntra,
            provider_subject: provider_subject.into(),
            tenant: Some(tenant.into()),
            display_name,
        }
    }

    pub fn atlassian(actor_id: ActorId, provider_subject: impl Into<String>, display_name: Option<String>) -> Self {
        Self {
            actor_id,
            provider: ExternalIdentityProvider::Atlassian,
            provider_subject: provider_subject.into(),
            tenant: None,
            display_name,
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
