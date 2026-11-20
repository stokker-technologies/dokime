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
use std::time::{Duration, Instant};

// Third Party Imports

// Local Imports
use super::Error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Step {
    pub stage: Stage,
    pub details: Vec<String>,
    pub duration: Duration,
    pub failed: bool,
}

#[derive(Default)]
pub struct Observer<'a> {
    report: Option<&'a mut (dyn FnMut(Step) + Send)>,
    active: Option<(Stage, Instant)>,
}

/* === Enums === */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Manifest,
    Kit,
    Appearance,
    Plugins,
    Bundle,
    Sign,
    Output,
}

/* === Traits === */

/* === Implementations === */

impl<'a> Observer<'a> {
    pub fn new(report: &'a mut (dyn FnMut(Step) + Send)) -> Self {
        Self { report: Some(report), active: None }
    }

    pub(super) fn begin(&mut self, stage: Stage) {
        self.active = Some((stage, Instant::now()));
    }

    pub(super) fn complete(&mut self, details: Vec<String>) {
        self.emit(details, false);
    }

    pub(super) fn failed(&mut self, error: &Error) {
        self.emit(vec![error.to_string()], true);
    }

    fn emit(&mut self, details: Vec<String>, failed: bool) {
        if let Some((stage, started)) = self.active.take()
            && let Some(report) = &mut self.report
        {
            report(Step {
                stage,
                details,
                duration: started.elapsed(),
                failed,
            });
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
