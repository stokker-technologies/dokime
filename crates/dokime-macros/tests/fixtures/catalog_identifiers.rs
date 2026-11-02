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

// Local Imports

/* === Constants === */

const POLICY: sdk::harmos::operations::OperationPolicy = sdk::harmos::operations::OperationPolicy {
    timeout: std::time::Duration::from_secs(5),
    attempt_timeout: std::time::Duration::from_secs(5),
    cancellation_timeout: std::time::Duration::ZERO,
    retries: sdk::harmos::operations::RetryPolicy::Never,
};

#[cfg(any(duplicate_suites, duplicate_templates))]
const SECOND_SUITE: &str = "checks";

#[cfg(not(any(duplicate_suites, duplicate_templates)))]
const SECOND_SUITE: &str = "other";

#[cfg(duplicate_templates)]
const SECOND_TEMPLATE: &str = "checks.probe";

#[cfg(duplicate_suites)]
const SECOND_TEMPLATE: &str = "checks.other";

#[cfg(not(any(duplicate_suites, duplicate_templates)))]
const SECOND_TEMPLATE: &str = "other.probe";

#[cfg(duplicate_records)]
const SECOND_RECORD: &str = "checks.record";

#[cfg(not(duplicate_records))]
const SECOND_RECORD: &str = "other.record";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct FirstTemplate;

struct SecondTemplate;

struct FirstSuite;

struct SecondSuite;

struct Plugin;

#[sdk::record(id = "checks.record", label = "First", severity = info)]
struct FirstRecord {
    value: u64,
}

#[sdk::record(id = SECOND_RECORD, label = "Second", severity = info)]
struct SecondRecord {
    value: u64,
}

#[sdk::stream(id = "checks.samples", label = "First", window = 8)]
struct FirstStream {
    value: u64,
}

#[cfg_attr(duplicate_streams, sdk::stream(id = "checks.samples", label = "Second", window = 8))]
#[cfg_attr(not(duplicate_streams), sdk::stream(id = "other.samples", label = "Second", window = 8))]
struct SecondStream {
    value: u64,
}

/* === Enums === */

#[sdk::catalog]
enum FirstTemplates {
    First(FirstTemplate),
}

#[sdk::catalog]
enum SecondTemplates {
    Second(SecondTemplate),
}

#[sdk::catalog]
enum Suites {
    First(FirstSuite),
    Second(SecondSuite),
}

#[sdk::catalog]
enum FirstRecords {
    First(FirstRecord),
}

#[sdk::catalog]
enum Records {
    #[nested]
    First(FirstRecords),

    Second(SecondRecord),
}

#[sdk::catalog]
enum Streams {
    First(FirstStream),
    Second(SecondStream),
}

#[sdk::catalog]
enum Operations {}

/* === Traits === */

/* === Implementations === */

#[sdk::template(id = "checks.probe", label = "First")]
impl FirstTemplate {
    async fn initialize(_: &mut sdk::execution::InitializationContext<'_>) -> Result<Self, std::convert::Infallible> {
        Ok(Self)
    }

    #[sdk::step(label = "Run")]
    fn run(&mut self, _: &mut sdk::execution::StepContext<'_>) -> Result<(), std::convert::Infallible> {
        Ok(())
    }
}

#[sdk::template(id = SECOND_TEMPLATE, label = "Second")]
impl SecondTemplate {
    async fn initialize(_: &mut sdk::execution::InitializationContext<'_>) -> Result<Self, std::convert::Infallible> {
        Ok(Self)
    }

    #[sdk::step(label = "Run")]
    fn run(&mut self, _: &mut sdk::execution::StepContext<'_>) -> Result<(), std::convert::Infallible> {
        Ok(())
    }
}

#[sdk::suite(id = "checks", label = "First", templates = FirstTemplates)]
impl FirstSuite {}

#[sdk::suite(id = SECOND_SUITE, label = "Second", templates = SecondTemplates)]
impl SecondSuite {}

impl sdk::harmos::builtins::lifecycle::Policy for Plugin {
    const INITIALIZATION_POLICY: sdk::harmos::operations::OperationPolicy = POLICY;
    const SHUTDOWN_POLICY: sdk::harmos::operations::OperationPolicy = POLICY;
}

#[sdk::plugin(
    id = "catalog_checks",
    requires(),
    contributions(sdk::Suites<Suites>, sdk::Records<Records>, sdk::Evidence<Streams>),
    restart = "never",
)]
impl sdk::Plugin for Plugin {
    const CATALOGS: &'static [sdk::harmos::Catalog] = &[];
}

impl sdk::harmos::Lifecycle for Plugin {
    type Config = sdk::plugin::Configuration;
    type Error = sdk::Refusal;
    type Context = sdk::Supervisor;
    async fn prepare(config: sdk::plugin::Configuration, setup: &mut sdk::harmos::Setup<Self>) -> Result<Self, sdk::Refusal> {
        let owner = { Self };
        setup.resource::<()>(owner.resources());
        sdk::execution::prepare::<Self, Suites>(setup)?;
        Ok(owner)
    }
}

impl Plugin {
    fn resources(&self) -> std::sync::Arc<()> {
        std::sync::Arc::new(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn main() {}

/* === Tests === */
