/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Rust Plugin Authoring and Typed Protocol Clients.

/* === Modules === */

pub mod client;
mod context;
mod data;
mod error;
pub mod host;
mod message;
mod operation;
mod plugin;
pub mod prelude;
mod prompt;
mod record;
mod stream;
mod suite;
mod template;

/* === Re-Exports === */

pub use client::{Client, ClientSettings};
pub use context::Context;
pub use data::FromValue;
pub use dokime_derive::{Message, Parameters, Record, Row, Settings};
pub use dokime_protocol::{builders, dokime as protocol, v1};
pub use error::{Error, Result};
pub use harmos::clock;
pub use harmos::{Classification, Failure, OperationPolicy, OperationRequest, Publisher, Visibility};
pub use operation::Operation;
pub use plugin::{CatalogTypes, Plugin};
pub use prompt::Prompt;
pub use record::Record;
pub use stream::{Observer, Stream};
pub use suite::Suite;
pub use template::{Criterion, Step, Template};

/* === Imports === */

// Standard Library Imports

// Third Party Imports
extern crate self as dokime;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

// Protobuf Data with the Harmos Descriptor Contract.
pub trait Message: harmos::Message {}

/* === Implementations === */

impl<T: harmos::Message> Message for T {}

/* === Public Functions === */

/* === Private Functions === */

// Dependencies used by the Harmos data derives, including when renamed by a consumer.
#[doc(hidden)]
pub mod encoding {
    /* === Modules === */

    /* === Re-Exports === */

    pub use harmos::Message as ContractMessage;
    pub use harmos::encoding::*;

    /* === Imports === */

    // Standard Library Imports

    // Third Party Imports

    // Local Imports

    /* === Constants === */

    /* === Macros === */

    /* === Type Aliases === */

    /* === Structs === */

    /* === Enums === */

    /* === Traits === */

    /* === Implementations === */

    /* === Public Functions === */

    /* === Private Functions === */

    /* === Tests === */
}

/* === Tests === */
