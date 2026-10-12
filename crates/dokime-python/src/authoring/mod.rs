/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod artifacts;
mod callbacks;
pub(crate) mod context;
mod traceability;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::collections::BTreeSet;
use std::sync::Arc;
// Third Party Imports
use async_trait::async_trait;
use harmos::sidecar::StreamDeclaration;
use pyo3::{
    prelude::*,
    types::{PyBool, PyDict},
};
use pyo3_async_runtimes::TaskLocals;
// Local Imports
use crate::{pyserde, to_value_error};
use context::Bridge;
use dokime::diagnostics::Diagnostic;
use dokime::domains::InstalledPlugin;
use dokime::execution::*;
use dokime_app::runtime::EmbeddedProvider;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct Provider {
    catalog: InstalledPlugin,
    streams: Vec<StreamDeclaration>,
    bindings: Vec<(String, Arc<Binding>)>,
    owner: Py<PyAny>,
    locals: TaskLocals,
    callbacks: Callbacks,
}

struct Binding {
    definition: TemplateDefinition,
    callback: Py<PyAny>,
    locals: TaskLocals,
    observers: Vec<DeclaredObserver>,
}

struct Template {
    instance: Py<PyAny>,
}

struct Own {
    value: Py<PyAny>,
    locals: TaskLocals,
    callbacks: Callbacks,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Provider {
    pub(crate) fn prepare(py: Python<'_>, owner: Py<PyAny>) -> PyResult<Self> {
        let locals = pyo3_async_runtimes::tokio::get_current_locals(py)?;
        let suites: dokime::definition::SuiteSet =
            pyserde::from_py(&owner.getattr(py, "suites")?.into_bound(py))?;
        let streams = pyserde::from_py(&owner.getattr(py, "streams")?.into_bound(py))?;
        let supplied = owner.getattr(py, "bindings")?;
        let supplied = supplied.bind(py).cast::<PyDict>()?;
        let mut bindings = Vec::new();
        let mut seen = BTreeSet::new();
        for suite in &suites.suites {
            let workflow = suite.workflow.as_deref().unwrap_or(&suite.suite_id);
            for template in &suite.suite.test_case_templates {
                let id = template.metadata.identifier.as_str();
                if !seen.insert(id.to_owned()) {
                    return Err(to_value_error(format!("duplicate Python template {id}")));
                }
                let callback = supplied
                    .get_item(id)?
                    .ok_or_else(|| to_value_error(format!("no callback binding for {id}")))?;
                // The Python binding still registers `(stream, label, interval)`
                // triples: a language host names its observers by the stream
                // they publish on, because it reads them out of a registered
                // catalog rather than off an authored function.
                let observers: Vec<(String, String, u64)> =
                    callback.getattr("samplers")?.extract()?;
                bindings.push((
                    workflow.to_owned(),
                    Arc::new(Binding {
                        definition: TemplateDefinition {
                            template: template.clone(),
                            records: suite
                                .record_contracts
                                .iter()
                                .filter(|contract| {
                                    contract
                                        .template_id
                                        .as_ref()
                                        .is_none_or(|id| id == &template.test_case_template_id)
                                })
                                .cloned()
                                .collect(),
                            streams: suite
                                .telemetry_contracts
                                .iter()
                                .filter(|contract| {
                                    contract
                                        .template_id
                                        .as_ref()
                                        .is_none_or(|id| id == &template.test_case_template_id)
                                })
                                .cloned()
                                .collect(),
                        },
                        callback: callback.unbind(),
                        locals: locals.clone(),
                        observers: observers
                            .into_iter()
                            .map(|(id, label, interval)| {
                                // The stream is what this observer publishes
                                // on, so it is declared beside it: that list is
                                // what admits the run's rows, and a language
                                // host has no authored `streams(...)` to read it
                                // off.
                                let stream = dokime::telemetry::TelemetryContract::declared(
                                    id.clone(),
                                    label.clone(),
                                );

                                DeclaredObserver::new(id, label, interval)
                                    .publishing(vec![stream], Vec::new())
                            })
                            .collect(),
                    }),
                ));
            }
        }
        if supplied.len() != seen.len() {
            return Err(to_value_error(
                "Python bindings must exactly match the registered templates",
            ));
        }

        Ok(Self {
            catalog: InstalledPlugin {
                source: "python".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                enabled: true,
                suites,
                ..Default::default()
            },
            streams,
            bindings,
            owner,
            locals,
            callbacks: Callbacks::default(),
        })
    }

    async fn lifecycle(&self, method: &str) -> dokime_app::Result<()> {
        let task = Python::attach(|py| {
            let coroutine = self.owner.call_method0(py, method)?;
            callbacks::Task::start(py, &self.locals, &self.callbacks, coroutine.into_bound(py))
        })
        .map_err(|error| dokime_app::Error::internal("python.provider", error.to_string()))?;
        let result = task
            .result()
            .await
            .map_err(|error| dokime_app::Error::internal("python.provider", error.to_string()))?;
        require_none(&result)
            .map_err(|error| dokime_app::Error::internal("python.provider", error.to_string()))
    }
}

#[async_trait]
impl EmbeddedProvider for Provider {
    fn catalog(&self) -> InstalledPlugin {
        self.catalog.clone()
    }
    fn streams(&self) -> Vec<StreamDeclaration> {
        self.streams.clone()
    }

    async fn start(&self) -> dokime_app::Result<Vec<Arc<dyn Runnable>>> {
        self.lifecycle("start").await?;
        Ok(self
            .bindings
            .iter()
            .map(|(workflow, binding)| {
                Arc::new(
                    Workflow::<Template>::declaring(
                        workflow,
                        &self.catalog.source,
                        binding.definition.clone(),
                        binding.observers.clone(),
                    )
                    .bound(binding.clone()),
                ) as Arc<dyn Runnable>
            })
            .collect())
    }

    async fn stop(&self) -> dokime_app::Result<()> {
        let failures = self.callbacks.settle().await;
        self.lifecycle("close").await?;
        if let Some(error) = failures.first() {
            return Err(dokime_app::Error::internal(
                "python.provider",
                error.to_string(),
            ));
        }

        Ok(())
    }
}

impl SidecarTemplate for Template {
    type Own = Own;
    type Parameters = ();

    const RUN: &'static str = "dokime.python.unbound.run";
    const SIGNAL: &'static str = "dokime.python.unbound.signal";

    fn definition() -> TemplateDefinition {
        unreachable!("a Python template is declared by its binding, not by its Rust type")
    }

    fn initialize<'a>(
        ctx: &'a mut InitializationContext<'_, Template, Own>,
    ) -> InitializationFuture<'a, (Self, Own)> {
        Box::pin(async move {
            let binding = ctx.environment().binding::<Binding>();
            let instance = context::call(
                binding,
                &binding.callback,
                "initialize",
                None,
                Bridge::Initialize(ctx),
            )
            .await?;
            let value = Python::attach(|py| instance.clone_ref(py));
            Ok((
                Self { instance },
                Own {
                    value,
                    locals: binding.locals.clone(),
                    callbacks: ctx.environment().callbacks().clone(),
                },
            ))
        })
    }

    fn traceability<'a>(
        &'a self,
        ctx: &'a TraceabilityContext<'_, Template, Own>,
    ) -> TraceabilityFuture<'a> {
        Box::pin(async move {
            let value = context::call(
                ctx.environment().binding::<Binding>(),
                &self.instance,
                "traceability",
                None,
                Bridge::Traceability(ctx),
            )
            .await?;
            Python::attach(|py| traceability::sections(value.bind(py)))
                .map_err(callbacks::diagnostic)
        })
    }

    fn run_step<'a>(
        &'a mut self,
        step: &'a str,
        ctx: &'a mut StepContext<'_, Template, Own>,
    ) -> StepFuture<'a> {
        Box::pin(async move {
            let value = context::call(
                ctx.environment().binding::<Binding>(),
                &self.instance,
                "step",
                Some(step),
                Bridge::Step(ctx),
            )
            .await?;
            require_none(&value)
        })
    }

    fn cleanup<'a>(&'a mut self, ctx: &'a mut CleanupContext<'_, Template, Own>) -> StepFuture<'a> {
        Box::pin(async move {
            let reason = format!("{:?}", ctx.reason()).to_lowercase();
            let value = context::call(
                ctx.environment().binding::<Binding>(),
                &self.instance,
                "cleanup",
                Some(&reason),
                Bridge::Cleanup(ctx),
            )
            .await?;
            require_none(&value)
        })
    }

    fn evaluate<'a>(
        criterion: &'a str,
        ctx: &'a CriterionContext<'_, Template, Self::Parameters>,
    ) -> EvaluationFuture<'a, bool> {
        let binding = ctx.binding::<Binding>();
        Box::pin(async move {
            let value = context::call(
                binding,
                &binding.callback,
                "criterion",
                Some(criterion),
                Bridge::Criterion(ctx),
            )
            .await?;
            Python::attach(|py| {
                if !value.bind(py).is_exact_instance_of::<PyBool>() {
                    return Err(Diagnostic::of("a Python criterion must return bool"));
                }
                value
                    .extract::<bool>(py)
                    .map(Some)
                    .map_err(callbacks::diagnostic)
            })
        })
    }

    fn observe<'a>(
        observer: &'a str,
        ctx: &'a mut ObserverContext<'_, Template, Own>,
    ) -> StepFuture<'a> {
        Box::pin(async move {
            let stream = match dokime::core::Identifier::new(observer) {
                Ok(stream) => stream,
                Err(refusal) => {
                    return Err(Diagnostic::of(format!(
                        "`{observer}` is not a valid stream identifier: {refusal}"
                    )));
                }
            };
            let tick = *ctx.tick();
            let task = {
                let own = ctx.resources();
                Python::attach(|py| {
                    let coroutine = own.value.call_method1(
                        py,
                        "sample",
                        (observer, tick.index(), tick.elapsed().as_secs_f64()),
                    )?;
                    callbacks::Task::start(
                        py,
                        &own.locals,
                        &own.callbacks,
                        coroutine.into_bound(py),
                    )
                })
                .map_err(callbacks::diagnostic)?
            };
            let value = task.result().await?;
            let row = Python::attach(|py| pyserde::value_map_from_py(value.bind(py)))
                .map_err(callbacks::diagnostic)?;
            ctx.publish_row(&stream, row).await;

            Ok(())
        })
    }
}

/* === Public Functions === */
#[pyo3::pymodule]
pub(crate) fn authoring(module: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
    use pyo3::prelude::*;

    module.add_class::<context::PyContext>()?;
    module.add_class::<artifacts::PyArtifacts>()?;
    module.add_class::<artifacts::PyArtifactWriter>()?;
    Ok(())
}

/* === Private Functions === */
fn require_none(value: &Py<PyAny>) -> Result<(), Diagnostic> {
    if Python::attach(|py| value.is_none(py)) {
        Ok(())
    } else {
        Err(Diagnostic::of(
            "a Python lifecycle callback or step must return None",
        ))
    }
}

/* === Tests === */
