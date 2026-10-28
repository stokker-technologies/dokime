/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


use sdk::execution::{InitializationContext, StepContext};

const FOREIGN: &str = "elsewhere.probe";


struct Foreign;

struct First;

struct Second;

struct Borrowed;

struct Repeated;

#[sdk::catalog]
enum BorrowedTemplates {
    Foreign(Foreign),
}

#[sdk::catalog]
enum RepeatedTemplates {
    First(First),
    Second(Second),
}


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


fn main() {}
