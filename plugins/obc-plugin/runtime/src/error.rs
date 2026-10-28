/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct LaunchError(#[from] dokime::harmos::runtime::Incompatible);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("testbench has stopped")]
    Stopped,

    #[error(transparent)]
    StateManager(#[from] dokime::harmos::Error),

    #[error("supply requires 0..300 V RMS, positive load/current limit, 1..1000 Hz and phase within (-90,90) degrees")]
    InvalidSupply,

    #[error("clear the latched fault before starting")]
    FaultLatched,
}
