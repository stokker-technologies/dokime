/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
#[allow(dead_code)]
mod one_prelude {
    use dokime_runtime::prelude::*;

    fn stream_adapters<S: Stream + StreamExt>() {}

    fn elapsed(_: Elapsed) {}

    fn stage_contexts(
        _initialization: &InitializationContext<'_>, _traceability: &TraceabilityContext<'_>, _step: &StepContext<'_>,
        _observer: &ObserverContext<'_>, _cleanup: &CleanupContext<'_>, _criterion: &CriterionContext<'_>,
    ) {
    }

    fn document_types(_: Document, _: Section, _: Plot, _: Line, _: Band) {}

    fn traceability_types(_: TraceabilityRow, _: TraceabilitySection) {}

    fn plugin_trait<P: Plugin>() {}

    fn supervisor(_: &Supervisor) {}

    fn refusal() -> Refusal {
        Refusal::of("refused")
    }

    fn configuration(config: Configuration) -> Result<NoSettings, Refusal> {
        config.settings::<NoSettings>().map_err(|error| Refusal::of(error.to_string()))
    }

    fn scope(_: &ExecutionScope) {}

    fn ticks(execution: &Tick) -> u64 {
        execution.index()
    }

    fn card_payloads(_: StatusResponse, _: OptionsResponse, _: Discovered, _: NoSettings) {}

    fn secret(_: Secret) {}

    fn publisher(_: &Publisher) {}

    fn slot<T>(_: &Slot<T>) {}

    fn host_module(_: &host::System) {}
}

#[allow(dead_code)]
mod bare_attributes {
    use dokime_runtime::prelude::*;

    const POLICY: dokime_runtime::harmos::operations::OperationPolicy = dokime_runtime::harmos::operations::OperationPolicy {
        timeout: std::time::Duration::from_secs(120),
        attempt_timeout: std::time::Duration::from_secs(120),
        retries: dokime_runtime::harmos::operations::RetryPolicy::Never,
    };

    #[message]
    #[derive(Clone, PartialEq)]
    pub(crate) struct GetReadingResponse {
        pub(crate) reads: u64,
    }
    #[settings]
    #[derive(Debug, Default, PartialEq)]
    struct BenchSettings {
        workspace: Workspace,
    }

    #[section(label = "Bench")]
    #[derive(Debug, Default, PartialEq)]
    struct Workspace {
        #[setting(label = "Port")]
        port: Option<String>,
    }

    struct Bench;

    #[catalog]
    enum Operations {
        GetReading(GetReading),
        GetStatus(GetStatus),
    }
    #[operation(id = "prelude_bench.get_reading", description = "Reads.", policy = POLICY, response = GetReadingResponse)]
    #[derive(Clone, PartialEq)]
    struct GetReading {}
    impl dokime_runtime::Operation for GetReading {
        type Error = Refusal;

        async fn run(
            self, _: dokime_runtime::harmos::Context<'_, Self>,
        ) -> Result<dokime_runtime::harmos::Response<GetReadingResponse>, Refusal> {
            let answered: Result<GetReadingResponse, Refusal> = async { Ok(GetReadingResponse { reads: 1 }) }.await;
            answered.map(Into::into)
        }
    }
    #[operation(id = "prelude_bench.get_status", description = "Reports.", policy = POLICY, response = StatusResponse)]
    #[derive(Clone, PartialEq)]
    struct GetStatus {}
    impl dokime_runtime::Operation for GetStatus {
        type Error = Refusal;

        async fn run(
            self, _: dokime_runtime::harmos::Context<'_, Self>,
        ) -> Result<dokime_runtime::harmos::Response<StatusResponse>, Refusal> {
            let answered: Result<StatusResponse, Refusal> = async { Ok(StatusResponse::ready()) }.await;
            answered.map(Into::into)
        }
    }

    impl dokime_runtime::harmos::builtins::lifecycle::Policy for Bench {
        const INITIALIZATION_POLICY: dokime_runtime::harmos::operations::OperationPolicy = POLICY;
        const SHUTDOWN_POLICY: dokime_runtime::harmos::operations::OperationPolicy = POLICY;
    }

    #[plugin(
        id = "prelude_bench",
        provides = Operations,
        requires(),
        contributions(dokime_runtime::Settings<BenchSettings>),
        restart = "never", description = "Bare attributes.")]
    impl Plugin for Bench {
        type Config = Configuration;
        type Error = Refusal;
        type Context = Supervisor;
        type Resources = ();

        fn prepare(config: Configuration) -> Result<Self, Refusal> {
            dokime_runtime::plugin::refusal(config.settings::<BenchSettings>())?;
            Ok(Self)
        }

        async fn initialize(&mut self, _: &Supervisor) -> Result<(), Refusal> {
            Ok(())
        }

        fn resources(&self) -> std::sync::Arc<()> {
            std::sync::Arc::new(())
        }

        fn bind_operations(
            &self, table: &mut dokime_runtime::harmos::operations::WireTable,
        ) -> Result<(), dokime_runtime::harmos::operations::ServingError> {
            table.register(Operations::registration({
                let mut dependencies = dokime_runtime::harmos::Dependencies::new();
                dependencies.insert(self.resources()).map_err(|_| dokime_runtime::harmos::sidecar::operations::ServingError::Catalog)?;
                dependencies
            }))
        }

        async fn stop(&mut self, _: &Supervisor) -> Result<(), Refusal> {
            Ok(())
        }

        async fn cleanup(&mut self) -> Result<(), Refusal> {
            Ok(())
        }
    }
}

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
