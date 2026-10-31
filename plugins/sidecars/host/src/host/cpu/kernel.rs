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
use std::{hint::black_box, time::Instant};

// Third Party Imports

// Local Imports

/* === Constants === */

// Fixed Arithmetic Defines The Capacity And Load Workload.
const ACCUMULATORS: usize = 8;
const OPERATIONS_PER_FMA: u64 = 2;
const INITIAL_VALUE: f64 = 1.0;
const MULTIPLIER: f64 = 1.000_000_000_000_001;
const ADDEND: f64 = 0.000_001;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Kernel {
    batch_iterations: u32,
    values: [f64; ACCUMULATORS],
    multiplier: f64,
    addend: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Kernel {
    pub(super) fn new(batch_iterations: u32) -> Self {
        Self {
            batch_iterations,
            values: black_box([INITIAL_VALUE; ACCUMULATORS]),
            multiplier: black_box(MULTIPLIER),
            addend: black_box(ADDEND),
        }
    }
    pub(super) fn batch(&mut self) -> u64 {
        for _ in 0..self.batch_iterations {
            for value in &mut self.values {
                *value = value.mul_add(self.multiplier, self.addend);
            }
        }
        black_box(&self.values);
        u64::from(self.batch_iterations) * ACCUMULATORS as u64 * OPERATIONS_PER_FMA
    }
    pub(super) fn until(&mut self, deadline: Instant) -> u64 {
        let mut operations = 0;
        while Instant::now() < deadline {
            operations += self.batch();
        }
        operations
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    #[test]
    fn counted_work_matches_accumulator_evolution() {
        let mut kernel = super::Kernel::new(4_096);
        for _ in 0..2 {
            let before = kernel.values[0];
            let operations = kernel.batch();
            let operations_per_iteration = kernel.values.len() as u64 * super::OPERATIONS_PER_FMA;
            assert!(operations > 0);
            assert_eq!(operations % operations_per_iteration, 0);
            let iterations = operations / operations_per_iteration;
            let expected = (0..iterations).fold(before, |value, _| value.mul_add(kernel.multiplier, kernel.addend));
            assert_eq!(kernel.values, [expected; super::ACCUMULATORS]);
        }
    }
}
