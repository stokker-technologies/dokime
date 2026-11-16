/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

//! Dokime contracts and the vendored Harmos transport, independent of the host runtime.

/* === Modules === */

pub mod builders;
pub mod host;
pub mod model;
pub mod plugin;

#[rustfmt::skip]
mod reflection;
#[rustfmt::skip]
mod sources;

// Checked-In Output of the Generator Exercised by the Integration Drift Check.
include!("generated/modules.rs");

/* === Re-Exports === */

pub use sources::SOURCES;

/* === Imports === */

// Standard Library Imports
use std::sync::OnceLock;

// Third Party Imports
use prost_reflect::DescriptorPool;

// Local Imports

/* === Constants === */

/// Authoritative Dokime descriptors, including their Harmos and Google imports.
pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("descriptors.bin");

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Shared descriptors for generated messages and host operation registration.
pub fn descriptor_pool() -> &'static DescriptorPool {
    static POOL: OnceLock<DescriptorPool> = OnceLock::new();
    POOL.get_or_init(|| DescriptorPool::decode(FILE_DESCRIPTOR_SET).expect("checked-in Dokime protocol descriptors"))
}

/* === Private Functions === */

/* === Tests === */
