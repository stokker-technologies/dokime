/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Finite Query Schema Contracts

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Schema {
    pub identity: String,
    pub versions: Vec<u32>,
    pub fields: Vec<Field>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub id: String,
    pub value_type: FieldType,
    pub nullable: bool,
    pub unit: Option<String>,
}

/* === Enums === */

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldType {
    Boolean,
    // Existing Integer Contracts Admit Both Int64 And Uint64, Preserved Per Value.
    Integer,
    Int64,
    Uint64,
    Float64,
    Text,
    Datetime,
    Severity,
    Object,
    Array,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
