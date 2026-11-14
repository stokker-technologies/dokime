/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model::{OperationContract, OperationPolicy, Visibility};
use prost::Message;
use prost_types::FileDescriptorSet;

// Local Imports
use crate::Result;
use dokime_protocol::plugin::{ExecutionsInitializeRequest, ExecutionsInitializeResponse};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Bind the shared payload shape to one authored Protected identity and policy.
pub fn declaration(id: &str, policy: OperationPolicy) -> Result<OperationContract> {
    let types = FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?;
    let mut contract = crate::declaration::<ExecutionsInitializeRequest, ExecutionsInitializeResponse>(id, types)?;
    contract.visibility = Visibility::Protected as i32;
    contract.policy = Some(policy);
    Ok(contract)
}

/* === Private Functions === */

/* === Tests === */
