/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// What a criterion reads its evidence as: the rows of one stream and the
// Harmos aggregate vocabulary over a column of them. The vocabulary and its
// mathematics are Harmos's own; Dokime names them here and adds nothing.

pub use crate::execution::evidence::Samples;
pub use harmos::Aggregate;
pub use harmos::signal::{Aggregates, Series, Summary};
