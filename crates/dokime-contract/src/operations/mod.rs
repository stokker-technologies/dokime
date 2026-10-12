/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod cursor;
pub mod runtime;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos::catalog::{Member, Visitor};
use harmos::operations::{WireDeclaration, WireOperation};
use harmos::schema::Described;
use serde::{Deserialize, Serialize};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct Endpoints;

pub struct Declaration {
    pub path: &'static str,
    pub access: Access,
    pub wire: WireDeclaration,
    pub rust_type: fn() -> &'static str,
}

/* === Enums === */
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Access {
    Read,
    Control,
    Admin,
}

#[harmos::catalog]
pub enum Operations {
    Health(runtime::GetHealth),
    Identity(runtime::GetIdentity),
}

/* === Traits === */
pub trait Endpoint: WireOperation {
    const PATH: &'static str;
    const ACCESS: Access;
}

/* === Implementations === */
impl Visitor for Endpoints {
    type Entry = Declaration;
}

impl<O> Member<O> for Endpoints
where
    O: Endpoint,
    O::Request: Described,
    O::Response: Described,
{
    const ENTRY: Declaration = Declaration {
        path: O::PATH,
        access: O::ACCESS,
        wire: WireDeclaration::of::<O>(),
        rust_type: std::any::type_name::<O>,
    };
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
