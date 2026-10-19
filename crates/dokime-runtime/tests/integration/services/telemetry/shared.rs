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
use crate::service_support::prelude::*;
use dokime_runtime::InstalledPlugin;
use dokime_runtime::definition::SuiteSet;
use dokime_runtime::execution::Runnable;
use dokime_runtime::runtime::EmbeddedProvider;
use dokime_runtime::schema::Described;
use harmos::sidecar::StreamDeclaration;
use harmos::storage::{
    operations::Query,
    query::{RowSlice, SchemaSelection, Source},
};

/* === Constants === */
const SOURCE: &str = "fixture.shared";
const SUITE: &str = "fixture.shared.checks";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime_runtime::stream(id = "fixture.shared.samples", window = 32, label = "Samples")]
struct Sample {
    value: f64,
}

struct Checks;

struct First;

struct Second;

struct SharedProvider {
    catalog: InstalledPlugin,
}

/* === Enums === */
#[dokime_runtime::catalog]
enum Streams {
    Sample(Sample),
}

#[dokime_runtime::catalog]
enum Templates {
    First(First),
    Second(Second),
}

/* === Traits === */

/* === Implementations === */
#[dokime_runtime::suite(id = "fixture.shared.checks", label = "Checks", templates = Templates)]
impl Checks {}

#[dokime_runtime::template(id = "fixture.shared.checks.first", label = "First", streams = Streams)]
impl First {
    async fn initialize(_: &mut dokime_runtime::execution::InitializationContext<'_>) -> dokime_runtime::Result<Self> {
        Ok(Self)
    }

    #[dokime_runtime::step(label = "Measure")]
    async fn measure(&mut self, ctx: &mut dokime_runtime::execution::StepContext<'_>) -> dokime_runtime::Result<()> {
        ctx.publish(Sample { value: 1.0 }).await;
        Ok(())
    }
}

#[dokime_runtime::template(id = "fixture.shared.checks.second", label = "Second", streams = Streams)]
impl Second {
    async fn initialize(_: &mut dokime_runtime::execution::InitializationContext<'_>) -> dokime_runtime::Result<Self> {
        Ok(Self)
    }

    #[dokime_runtime::step(label = "Measure")]
    async fn measure(&mut self, ctx: &mut dokime_runtime::execution::StepContext<'_>) -> dokime_runtime::Result<()> {
        ctx.publish(Sample { value: 2.0 }).await;
        Ok(())
    }
}

impl SharedProvider {
    fn new() -> Self {
        let evidence = dokime_runtime::catalog::entries::<Streams, dokime_runtime::catalog::StreamContracts>()
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

    async fn start(&self) -> dokime_runtime::runtime::Result<Vec<Arc<dyn Runnable>>> {
        Ok(vec![
            Arc::new(dokime_runtime::execution::Workflow::<First>::new(SUITE, SOURCE)) as Arc<dyn Runnable>,
            Arc::new(dokime_runtime::execution::Workflow::<Second>::new(SUITE, SOURCE)),
        ])
    }

    async fn stop(&self) -> dokime_runtime::runtime::Result<()> {
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn shared_stream_refusal(detail: &str) -> bool {
    detail.contains("telemetry definition fixture.shared.samples is already registered")
}

/* === Tests === */
#[tokio::test]
async fn shared_stream_publishes() {
    let root = tempdir().unwrap();
    let config = config(root.path());
    let mut runtime = Runtime::start(config.register(Arc::new(SharedProvider::new()))).await.unwrap();
    let handle = runtime.handle();
    let suite = Checks::definition().suite;

    // Run Both Templates
    let cycle = start_cases(&handle, metadata("fixture.shared.cycle", "Shared Cycle"), materialize_cases(SUITE, &suite), false).await;
    let receipt = handle.start_next_execution(cycle.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
    wait_for_cycle_lifecycle(&handle, &cycle, CycleStatus::Completed).await;

    // Each Execution Published Under Its Own Template
    let rows = handle
        .collect_telemetry(&TelemetrySelection {
            streams: vec![Identifier::new(Sample::ID).unwrap().into()],
            scope: ScopeSelection { test_cycle_id: Some(cycle.clone()), ..Default::default() },
            ..Default::default()
        })
        .await;
    let values = rows.iter().map(|row| row.values["value"].clone()).collect::<Vec<_>>();
    assert_eq!(rows.len(), 2, "both templates publish the shared stream");
    assert!(values.contains(&Value::Number(1.0)));
    assert!(values.contains(&Value::Number(2.0)));
    let templates = rows.iter().map(|row| row.scope.as_ref().and_then(|scope| scope.template_id.clone())).collect::<BTreeSet<_>>();
    assert_eq!(templates.len(), 2, "each row keeps its own template scope");
    let ended = handle.cycles().await.into_iter().find(|item| item.test_cycle_id == cycle).unwrap();
    assert!(
        ended.test_executions.iter().all(|execution| execution.status == dokime_domain::model::execution::result::ExecutionStatus::Passed)
    );

    // Run Admission Registers The Embedded Stream; Publication Only Appends Rows.
    let contracts = handle.telemetry_contracts().await;
    let stream = Identifier::new(Sample::ID).unwrap();
    let contract = contracts.into_iter().find(|contract| contract.stream == stream);
    let contract = contract.unwrap();
    let codec = dokime_runtime::storage::telemetry::codec(&contract).unwrap();
    let storage = handle.storage();
    let at = storage.flush().await.unwrap();
    let schema = codec.schema().clone();
    let query =
        Query { at, source: Source::Schemas(SchemaSelection { schema: Some(schema) }), transforms: Vec::new(), slice: RowSlice::default() };
    let mut schemas = storage.query(query).await.unwrap();
    while schemas.next().await.unwrap().is_some() {}
    assert_eq!(schemas.completion().unwrap().rows, 1, "neither template republishes the admitted schema");

    drop(handle);
    runtime.shutdown().await.unwrap();
}

#[tokio::test]
async fn shared_schema_refused() {
    let root = tempdir().unwrap();
    let mut runtime = Runtime::start(config(root.path())).await.unwrap();
    let mut provider = SharedProvider::new();
    let second = <Second as dokime_runtime::definition::DeclaredTemplate>::template_id();
    let suite = &mut provider.catalog.suites.suites[0];
    for contract in &mut suite.telemetry_contracts {
        if contract.template_id.as_ref() == Some(&second) {
            contract.columns[0].required = !contract.columns[0].required;
        }
    }

    let refused = runtime.register(Arc::new(provider)).await;

    assert!(refused.as_ref().is_err_and(|error| shared_stream_refusal(&error.to_string())), "{refused:?}");
    runtime.shutdown().await.unwrap();
}

#[tokio::test]
async fn shared_owner_refused() {
    let root = tempdir().unwrap();
    let mut runtime = Runtime::start(config(root.path()).register(Arc::new(SharedProvider::new()))).await.unwrap();
    let handle = runtime.handle();
    let mut definition = Checks::definition();
    definition.suite_id = "fixture.other.checks".to_owned();
    definition.suite.metadata = metadata("fixture.other.checks", "Other Checks");
    for template in &mut definition.suite.test_case_templates {
        template.test_case_template_id = TemplateId::from_seed(format!("other.{}", template.test_case_template_id));
    }
    for contract in &mut definition.telemetry_contracts {
        contract.template_id = contract.template_id.as_ref().map(|id| TemplateId::from_seed(format!("other.{id}")));
    }

    let refused = handle.set_suites("fixture.other", SuiteSet::new([definition])).await;

    assert!(refused.as_ref().is_err_and(|error| shared_stream_refusal(&error.to_string())), "{refused:?}");
    drop(handle);
    runtime.shutdown().await.unwrap();
}
