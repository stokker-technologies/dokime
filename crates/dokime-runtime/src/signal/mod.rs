/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Dokime-Owned Statistics And Signal Processing.

/* === Modules === */

mod aggregates;
#[cfg(feature = "fft")]
mod error;
#[cfg(feature = "native")]
mod reduce;
mod series;
#[cfg(feature = "fft")]
mod spectrum;
mod summary;

/* === Re-Exports === */

pub use aggregates::Aggregates;
#[cfg(feature = "fft")]
pub(crate) use error::{Error, Result};
#[cfg(feature = "native")]
pub(crate) use reduce::{decimate, decimate_indices, extrema, lttb, m4};
pub use series::Series;
#[cfg(feature = "fft")]
pub(crate) use spectrum::{FftBuffer, fft_in_place};
pub use summary::Summary;

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
