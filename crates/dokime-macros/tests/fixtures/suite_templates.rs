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
use sdk::execution::{InitializationContext, StepContext};
// Local Imports

/* === Constants === */
const FOREIGN: &str = "elsewhere.probe";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Foreign;

struct First;

struct Second;

struct Borrowed;

struct Repeated;

/* === Enums === */
#[sdk::catalog]
enum BorrowedTemplates {
    Foreign(Foreign),
}

#[sdk::catalog]
enum RepeatedTemplates {
    First(First),
    Second(Second),
}

/* === Traits === */

/* === Implementations === */
// A constant id is checked by the suite that owns it, not on its own line.
#[sdk::template(id = FOREIGN, label = "Foreign")]
impl Foreign {
    async fn initialize(
        _: &mut InitializationContext<'_>,
    ) -> Result<Self, std::convert::Infallible> {
        Ok(Self)
    }

    #[sdk::step(label = "Run")]
    fn run(&mut self, _: &mut StepContext<'_>) -> Result<(), std::convert::Infallible> {
        Ok(())
    }
}

#[sdk::template(id = "repeated.checks.probe", label = "First")]
impl First {
    async fn initialize(
        _: &mut InitializationContext<'_>,
    ) -> Result<Self, std::convert::Infallible> {
        Ok(Self)
    }

    #[sdk::step(label = "Run")]
    fn run(&mut self, _: &mut StepContext<'_>) -> Result<(), std::convert::Infallible> {
        Ok(())
    }
}

#[sdk::template(id = "repeated.checks.probe", label = "Second")]
impl Second {
    async fn initialize(
        _: &mut InitializationContext<'_>,
    ) -> Result<Self, std::convert::Infallible> {
        Ok(Self)
    }

    #[sdk::step(label = "Run")]
    fn run(&mut self, _: &mut StepContext<'_>) -> Result<(), std::convert::Infallible> {
        Ok(())
    }
}

#[sdk::suite(id = "borrowed.checks", label = "Borrowed", templates = BorrowedTemplates)]
impl Borrowed {}

#[sdk::suite(id = "repeated.checks", label = "Repeated", templates = RepeatedTemplates)]
impl Repeated {}

/* === Public Functions === */

/* === Private Functions === */
fn main() {}

/* === Tests === */
