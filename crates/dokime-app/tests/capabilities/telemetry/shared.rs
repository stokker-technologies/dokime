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
use std::collections::BTreeSet;
// Third Party Imports
use async_trait::async_trait;
// Local Imports
use crate::support::prelude::*;
use dokime::execution::Runnable;
use dokime::schema::Described;
use dokime_app::InstalledPlugin;
use dokime_app::definition::SuiteSet;
use dokime_app::runtime::EmbeddedProvider;
use harmos::sidecar::StreamDeclaration;

/* === Constants === */
const SOURCE: &str = "fixture.shared";
const SUITE: &str = "fixture.shared.checks";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime::stream(id = "fixture.shared.samples", window = 32, label = "Samples")]
struct Sample {
    value: f64,
}

struct Checks;

struct First;

struct Second;

/// One plugin whose two templates and plugin-level evidence all declare the
/// same stream, the way an author shares one row type across templates.
struct SharedProvider {
    catalog: InstalledPlugin,
}

/* === Enums === */
#[dokime::catalog]
enum Streams {
    Sample(Sample),
}

#[dokime::catalog]
enum Templates {
    First(First),
    Second(Second),
}

/* === Traits === */

/* === Implementations === */
#[dokime::suite(id = "fixture.shared.checks", label = "Checks", templates = Templates)]
impl Checks {}

#[dokime::template(id = "fixture.shared.checks.first", label = "First", streams = Streams)]
impl First {
    async fn initialize(
        _: &mut dokime::execution::InitializationContext<'_>,
    ) -> dokime::Result<Self> {
        Ok(Self)
    }

    #[dokime::step(label = "Measure")]
    async fn measure(
        &mut self,
        ctx: &mut dokime::execution::StepContext<'_>,
    ) -> dokime::Result<()> {
        ctx.publish(Sample { value: 1.0 }).await;
        Ok(())
    }
}

#[dokime::template(id = "fixture.shared.checks.second", label = "Second", streams = Streams)]
impl Second {
    async fn initialize(
        _: &mut dokime::execution::InitializationContext<'_>,
    ) -> dokime::Result<Self> {
        Ok(Self)
    }

    #[dokime::step(label = "Measure")]
    async fn measure(
        &mut self,
        ctx: &mut dokime::execution::StepContext<'_>,
    ) -> dokime::Result<()> {
        ctx.publish(Sample { value: 2.0 }).await;
        Ok(())
    }
}

impl SharedProvider {
    fn new() -> Self {
        let evidence = dokime::catalog::entries::<Streams, dokime::catalog::StreamContracts>()
            .into_iter()
            .map(|contract| contract())
            .collect();
        Self {
            catalog: InstalledPlugin {
                source: SOURCE.into(),
                version: "0.1.0".into(),
                enabled: true,
                suites: SuiteSet::new([Checks::definition()]),
                telemetry_contracts: evidence,
                ..Default::default()
            },
        }
    }
}

#[async_trait]
impl EmbeddedProvider for SharedProvider {
    fn catalog(&self) -> InstalledPlugin {
        self.catalog.clone()
    }

    fn streams(&self) -> Vec<StreamDeclaration> {
        vec![StreamDeclaration {
            metadata: <Sample as harmos::StreamSpec>::metadata(),
            row_metadata: <Sample as harmos::RowMetadata>::row_metadata(),
            id: Sample::ID.into(),
            schema: Sample::VERSION,
            window: Sample::WINDOW,
            row: harmos::schema::graph::Schema::from_declaration(
                <Sample as Described>::DEFINITION,
                &harmos::config::SchemaSettings::default(),
            )
            .unwrap(),
        }]
    }

    async fn start(&self) -> dokime_app::Result<Vec<Arc<dyn Runnable>>> {
        Ok(vec![
            Arc::new(dokime::execution::Workflow::<First>::new(SUITE, SOURCE)) as Arc<dyn Runnable>,
            Arc::new(dokime::execution::Workflow::<Second>::new(SUITE, SOURCE)),
        ])
    }

    async fn stop(&self) -> dokime_app::Result<()> {
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn shared_stream_refusal(detail: &str) -> bool {
    detail.contains("telemetry definition fixture.shared.samples is already registered")
}

/* === Tests === */
pub(crate) async fn shared_stream_publishes() {
    let root = tempdir().unwrap();
    let config = config(root.path());
    let mut runtime = Runtime::start(config.register(Arc::new(SharedProvider::new())))
        .await
        .unwrap();
    let handle = runtime.handle();
    let suite = Checks::definition().suite;

    // Run Both Templates
    let cycle = handle
        .start_cycle(
            CycleDefinition {
                suite: suite.clone(),
                metadata: metadata("fixture.shared.cycle", "Shared Cycle"),
                cases: materialize_cases(SUITE, &suite),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let receipt = handle
        .run_cycle_cases(cycle.clone(), CommandContext::default())
        .await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
    wait_for_cycle_lifecycle(&handle, &cycle, CycleStatus::Completed).await;

    // Each Execution Published Under Its Own Template
    let rows = handle
        .collect_telemetry(&TelemetrySelection {
            streams: vec![Identifier::new(Sample::ID).unwrap()],
            scope: ScopeSelection {
                test_cycle_id: Some(cycle.clone()),
                ..Default::default()
            },
            ..Default::default()
        })
        .await;
    let values = rows
        .iter()
        .map(|row| row.values["value"].clone())
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 2, "both templates publish the shared stream");
    assert!(values.contains(&Value::Number(1.0)));
    assert!(values.contains(&Value::Number(2.0)));
    let templates = rows
        .iter()
        .map(|row| {
            row.scope
                .as_ref()
                .and_then(|scope| scope.template_id.clone())
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(templates.len(), 2, "each row keeps its own template scope");
    let ended = handle
        .cycles()
        .await
        .into_iter()
        .find(|item| item.test_cycle_id == cycle)
        .unwrap();
    assert!(
        ended
            .test_executions
            .iter()
            .all(|execution| execution.status == dokime_app::execution::ExecutionStatus::Passed)
    );

    drop(handle);
    runtime.shutdown().await.unwrap();
}

pub(crate) async fn shared_stream_schema_refused() {
    let root = tempdir().unwrap();
    let mut runtime = Runtime::start(config(root.path())).await.unwrap();
    let mut provider = SharedProvider::new();
    let second = <Second as dokime::definition::DeclaredTemplate>::template_id();
    let suite = &mut provider.catalog.suites.suites[0];
    for contract in &mut suite.telemetry_contracts {
        if contract.template_id.as_ref() == Some(&second) {
            contract.columns[0].required = !contract.columns[0].required;
        }
    }

    let refused = runtime.register(Arc::new(provider)).await;

    assert!(
        refused
            .as_ref()
            .is_err_and(|error| shared_stream_refusal(&error.to_string())),
        "{refused:?}"
    );
    runtime.shutdown().await.unwrap();
}

pub(crate) async fn shared_stream_owner_refused() {
    let root = tempdir().unwrap();
    let mut runtime = Runtime::start(config(root.path()).register(Arc::new(SharedProvider::new())))
        .await
        .unwrap();
    let handle = runtime.handle();
    let mut definition = Checks::definition();
    definition.suite_id = "fixture.other.checks".to_owned();
    definition.suite.metadata = metadata("fixture.other.checks", "Other Checks");
    for template in &mut definition.suite.test_case_templates {
        template.test_case_template_id =
            TemplateId::from_seed(format!("other.{}", template.test_case_template_id));
    }
    for contract in &mut definition.telemetry_contracts {
        contract.template_id = contract
            .template_id
            .as_ref()
            .map(|id| TemplateId::from_seed(format!("other.{id}")));
    }

    let refused = handle
        .set_suites("fixture.other", SuiteSet::new([definition]))
        .await;

    assert!(
        refused
            .as_ref()
            .is_err_and(|error| shared_stream_refusal(&error.to_string())),
        "{refused:?}"
    );
    drop(handle);
    runtime.shutdown().await.unwrap();
}
