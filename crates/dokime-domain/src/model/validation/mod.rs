/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(crate) mod diagnostic;
pub mod evaluation;

pub use evaluation::CompatibilityStatus;
pub use evaluation::ConditionEvaluation;
pub use evaluation::ValidationMode;
