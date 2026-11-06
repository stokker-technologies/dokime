/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Prove Shared Template Operation Invocation

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_runtime::execution::{InitializationContext, StepContext};
use dokime_runtime::plugin::testing::Harness;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Checks;
struct Probe;

#[dokime_runtime::harmos::message]
struct Reading {
    value: u64,
}

#[dokime_runtime::harmos::operation(harmos_path = dokime_runtime::harmos, id = "invocation.read", response = Reading,
    policy = dokime_runtime::harmos::OperationPolicy {
        timeout: std::time::Duration::from_secs(2),
        attempt_timeout: std::time::Duration::from_secs(1),
        cancellation_timeout: std::time::Duration::ZERO,
        retries: dokime_runtime::harmos::RetryPolicy::Never,
    }, visibility = Public)]
struct Read {
    value: u64,
}

/* === Enums === */

#[dokime_runtime::catalog]
enum Required {
    Read(Read),
}

#[dokime_runtime::catalog]
enum Templates {
    Probe(Probe),
}

#[dokime_runtime::catalog]
enum Suites {
    Checks(Checks),
}

/* === Traits === */

/* === Implementations === */

#[dokime_runtime::suite(id = "invocation.checks", label = "Invocation", templates = Templates)]
impl Checks {}

#[dokime_runtime::template(id = "invocation.checks.probe", label = "Probe", requires = Required)]
impl Probe {
    async fn initialize(_: &(), _: &mut InitializationContext<'_>) -> Result<Self, dokime_runtime::Error> {
        Ok(Self)
    }

    #[dokime_runtime::step(label = "Read A Value")]
    async fn read(&mut self, ctx: &mut StepContext<'_>) -> Result<(), dokime_runtime::Error> {
        let response = ctx.operation(Read { value: 42 }).await?;
        assert_eq!(response.value, 43);
        let value = response.into_owned();
        assert_eq!(value.value, 43);
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn template_operation_preserves_harness_dispatch_and_response_ownership() {
    let report = Harness::template("invocation.checks.probe")
        .answering::<Read, _>(|request: Read| Reading { value: request.value + 1 })
        .run::<Suites>()
        .await
        .unwrap();
    assert!(report.passed(), "{}", report.diagnostics());
}
