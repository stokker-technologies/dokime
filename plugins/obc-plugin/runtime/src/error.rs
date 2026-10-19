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

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct LaunchError(#[from] harmos::runtime::Incompatible);

/* === Enums === */

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("testbench has stopped")]
    Stopped,

    #[error(transparent)]
    StateManager(#[from] harmos::Error),

    #[error("supply requires 0..300 V RMS, positive load/current limit, 1..1000 Hz and phase within (-90,90) degrees")]
    InvalidSupply,

    #[error("clear the latched fault before starting")]
    FaultLatched,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
