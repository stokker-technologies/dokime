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

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Kernel {
    values: [f64; 8],
    multiplier: f64,
    addend: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Kernel {
    pub(super) fn new() -> Self {
        Self {
            values: black_box([1.0; 8]),
            multiplier: black_box(1.000_000_000_000_001),
            addend: black_box(0.000_001),
        }
    }
    pub(super) fn batch(&mut self) -> u64 {
        for _ in 0..4096 {
            for value in &mut self.values {
                *value = value.mul_add(self.multiplier, self.addend);
            }
        }
        black_box(&self.values);
        4096 * 8 * 2
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
        let mut kernel = super::Kernel::new();
        for _ in 0..2 {
            let before = kernel.values[0];
            let operations = kernel.batch();
            let operations_per_iteration = kernel.values.len() as u64 * 2;
            assert!(operations > 0);
            assert_eq!(operations % operations_per_iteration, 0);
            let iterations = operations / operations_per_iteration;
            let expected = (0..iterations).fold(before, |value, _| value.mul_add(kernel.multiplier, kernel.addend));
            assert_eq!(kernel.values, [expected; 8]);
        }
    }
}
