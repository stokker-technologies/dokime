/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Document Admission And Materialization Budgets.

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

#[derive(Clone, Copy, Debug)]
pub struct DocumentLimits {
    pub source_bytes: usize,
    pub depth: usize,
    pub expanded_nodes: usize,
    pub materialized_bytes: usize,
    pub layout_depth: usize,
    pub unavailable_reason_bytes: usize,
    pub maximum_plot_points: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Default for DocumentLimits {
    fn default() -> Self {
        Self {
            source_bytes: 16777216,
            depth: 32,
            expanded_nodes: 4096,
            materialized_bytes: 33554432,
            layout_depth: 12,
            unavailable_reason_bytes: 512,
            maximum_plot_points: 100000,
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
