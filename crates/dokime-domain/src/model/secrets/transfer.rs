/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded Encrypted Secret Transfer And Explicit Import Choices.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::catalog::Metadata;

/* === Constants === */

pub const MAX_PLAINTEXT_BYTES: usize = 512 * 1024;
pub const MAX_CIPHERTEXT_BYTES: usize = MAX_PLAINTEXT_BYTES + 4096;
pub const MAX_PASSWORD_BYTES: usize = 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos_domain::message(skip_debug)]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Archive {
    pub ciphertext: Vec<u8>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Preview {
    pub host_id: String,
    pub host_label: String,
    pub generation: u64,
    pub entries: Vec<Entry>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub metadata: Metadata,
    pub exists: bool,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Choice {
    pub name: String,
    pub action: Action,
}

/* === Enums === */

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Replace,
    Skip,
}

/* === Traits === */

/* === Implementations === */

impl std::fmt::Debug for Archive {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("EncryptedSecretArchive(<redacted>)")
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
