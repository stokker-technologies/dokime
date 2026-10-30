/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Settle Execution Publications, Cancellation And Capture Leases.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::{
    CleanupStage, ExecutionCleanup, ExecutionFixtureContext, ExecutionSignal, Finalization, HostCapture, Identifier, Instant,
    ProgressCleanup, ProviderExecutionFailure, StepLeaseBridge, TestExecution, close_record_subscriptions, failure, sampling_scope,
};
use crate::domains::telemetry::DeclaredLeases;
use dokime_domain::model::execution::health::HealthSeverity;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Drop for ProgressCleanup {
    fn drop(&mut self) {
        self.registry.clear(&self.execution);
    }
}

impl Finalization {
    pub(crate) async fn pending(&self) -> bool {
        self.active.lock().await.is_some()
    }

    pub(super) async fn register(&self, cleanup: ExecutionCleanup) {
        let mut active = self.active.lock().await;
        assert!(active.is_none(), "previous execution cleanup remains owned");
        *active = Some(cleanup);
    }

    pub(super) async fn cancelling(&self, deadline: tokio::time::Instant) {
        if let Some(cleanup) = self.active.lock().await.as_mut() {
            cleanup.stop_deadline = Some(deadline);
        }
    }

    pub(super) async fn settled(&self) {
        if let Some(cleanup) = self.active.lock().await.as_mut() {
            cleanup.settled = true;
        }
    }

    pub(crate) async fn finish(&self, context: &ExecutionFixtureContext) -> Result<(), ProviderExecutionFailure> {
        let finalization = self.clone();
        let context = context.clone();
        tokio::spawn(async move {
            let mut active = finalization.active.lock().await;
            if let Some(cleanup) = active.as_mut() {
                cleanup.finish(&context).await?;
            }
            *active = None;
            Ok(())
        })
        .await
        .map_err(|error| failure("sidecar_cleanup", error.to_string()))?
    }
}

impl ExecutionCleanup {
    pub(super) async fn finish(&mut self, context: &ExecutionFixtureContext) -> Result<(), ProviderExecutionFailure> {
        if self.stage == CleanupStage::Settle {
            self.settle(context).await;
            self.stage = CleanupStage::Effects;
        }
        if self.stage == CleanupStage::Effects {
            #[cfg(feature = "testing")]
            super::testing::record(&self.execution.test_execution_id, "effects");
            context.runtime.restore_effects(Some(self.execution.test_execution_id.as_str())).await.map_err(telemetry_failure)?;
            self.stage = CleanupStage::Publications;
        }

        if self.stage == CleanupStage::Publications {
            #[cfg(feature = "testing")]
            super::testing::record(&self.execution.test_execution_id, "publications");
            let publication_started = Instant::now();
            tracing::debug!(execution_id = %self.execution.test_execution_id, "execution publication flush started");
            context.runtime.flush_plugin_publications().await.map_err(telemetry_failure)?;
            tracing::debug!(execution_id = %self.execution.test_execution_id,
            elapsed_ms = publication_started.elapsed().as_millis() as u64, "execution publication flush settled");
            self.stage = CleanupStage::Records;
        }
        if self.stage == CleanupStage::Records {
            #[cfg(feature = "testing")]
            super::testing::record(&self.execution.test_execution_id, "records");
            close_record_subscriptions(context, &self.execution).await?;
            self.stage = CleanupStage::Prompts;
        }
        if self.stage == CleanupStage::Prompts {
            self.session.prompts.lock().await.settle_on_cancel(context).await;
            self.stage = CleanupStage::Steps;
        }

        let capture_started = Instant::now();
        tracing::debug!(execution_id = %self.execution.test_execution_id, "execution capture release started");
        if self.stage == CleanupStage::Steps {
            #[cfg(feature = "testing")]
            super::testing::record(&self.execution.test_execution_id, "step_leases");
            self.session.step_leases.release_all(context).await?;
            self.stage = CleanupStage::Capture;
        }
        if self.stage == CleanupStage::Capture {
            #[cfg(feature = "testing")]
            super::testing::record(&self.execution.test_execution_id, "capture_leases");
            self.capture.release(context).await?;
            self.stage = CleanupStage::Complete;
        }
        tracing::debug!(execution_id = %self.execution.test_execution_id,
            elapsed_ms = capture_started.elapsed().as_millis() as u64, "execution capture release settled");
        Ok(())
    }

    async fn settle(&mut self, context: &ExecutionFixtureContext) {
        if self.settled {
            return;
        }
        self.session.prompts.lock().await.settle_on_cancel(context).await;
        let deadline = || {
            let now = tokio::time::Instant::now();
            now + dokime_operations::STOP_SETTLEMENT.timeout
        };
        let deadline = *self.stop_deadline.get_or_insert_with(deadline);
        let signal = self.session.target.signal(ExecutionSignal {
            execution_token: self.execution.test_execution_id.to_string(),
            cancel: true,
            ..ExecutionSignal::default()
        });
        match tokio::time::timeout_at(deadline, signal).await {
            Ok(Ok(_)) => {}
            Ok(Err(error)) => {
                tracing::debug!(detail = %error.message, "an unsettled run did not take its cancellation");
            }
            Err(_) => {
                tracing::debug!("an unsettled run exhausted its cancellation budget");
            }
        }
        self.settled = true;
    }
}

impl StepLeaseBridge {
    pub(super) async fn request(
        &self, context: &ExecutionFixtureContext, execution: &TestExecution, source: &str, request: crate::telemetry::LeaseRequest,
    ) -> crate::transport::LeaseResolution {
        let answer = context.runtime.lease_streams(sampling_scope(execution), std::slice::from_ref(&request.stream), source).await;
        let leases = match answer {
            Ok(leases) => leases,
            Err(error) => {
                return crate::transport::LeaseResolution {
                    granted: false,
                    message: error.to_string(),
                };
            }
        };
        let Some(lease_id) = leases.first() else {
            let message = format!("the host took no hold on {}", request.stream);
            return crate::transport::LeaseResolution { granted: false, message };
        };
        let mut held = self.held.lock().await;
        held.insert(request.lease_token.clone(), lease_id.clone());
        drop(held);
        drop(leases);
        crate::transport::LeaseResolution { granted: true, message: String::new() }
    }

    pub(super) async fn release(&self, context: &ExecutionFixtureContext, released: &[String]) -> Result<(), ProviderExecutionFailure> {
        if released.is_empty() {
            return Ok(());
        }
        let held = self.held.lock().await;
        let selected = released.iter().filter_map(|token| {
            let lease = held.get(token)?;
            let pair = (token.clone(), lease.clone());
            Some(pair)
        });
        let leases = selected.collect::<Vec<_>>();
        drop(held);
        self.release_owned(context, leases).await
    }

    pub(super) async fn release_all(&self, context: &ExecutionFixtureContext) -> Result<(), ProviderExecutionFailure> {
        let held = self.held.lock().await;
        let pairs = held.iter().map(|(token, lease)| {
            let token = token.clone();
            let lease = lease.clone();
            (token, lease)
        });
        let leases = pairs.collect::<Vec<_>>();
        drop(held);
        self.release_owned(context, leases).await
    }

    pub(super) async fn release_owned(
        &self, context: &ExecutionFixtureContext, leases: Vec<(String, String)>,
    ) -> Result<(), ProviderExecutionFailure> {
        if leases.is_empty() {
            return Ok(());
        }
        let lease_ids = leases.iter().map(lease_id).collect();
        context.runtime.release_stream_leases(lease_ids).await.map_err(lease_failure)?;
        let mut held = self.held.lock().await;
        for (token, lease) in leases {
            remove_matching_lease(&mut held, &token, &lease);
        }
        Ok(())
    }
}

impl HostCapture {
    pub(super) async fn start(context: &ExecutionFixtureContext, execution: &TestExecution, streams: &[Identifier], source: &str) -> Self {
        if streams.is_empty() {
            return Self::default();
        }
        // Each Stream Leases On Its Own, So One Refusal Never Leaves The Others Unsampled.
        let declared = match context.runtime.lease_declared_streams(sampling_scope(execution), streams, source).await {
            Ok(declared) => declared,
            Err(error) => {
                let reason = error.to_string();
                let refused = streams.iter().map(|stream| (stream.clone(), reason.clone())).collect();
                DeclaredLeases { refused, ..DeclaredLeases::default() }
            }
        };
        for (stream, reason) in &declared.refused {
            refuse_stream(context, execution, source, stream, reason).await;
        }
        Self { leases: declared.leases }
    }

    pub(super) async fn release(&mut self, context: &ExecutionFixtureContext) -> Result<(), ProviderExecutionFailure> {
        context.runtime.release_stream_leases(self.leases.clone()).await.map_err(lease_failure)?;
        self.leases.clear();
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

// A Refused Stream Is Told To The Operator As The Execution's Own Health Item, Not Only Logged.
async fn refuse_stream(context: &ExecutionFixtureContext, execution: &TestExecution, source: &str, stream: &Identifier, reason: &str) {
    tracing::warn!(plugin_id = source, stream = stream.as_str(), reason, "a declared stream could not be leased");
    let scope = execution
        .scope
        .clone()
        .with_cycle(execution.test_cycle_id.clone())
        .with_test_case(execution.test_case_id.clone())
        .with_test_execution(execution.test_execution_id.clone());
    let message = format!("Telemetry stream {stream} is not sampled for this run");
    let recorded = context.runtime.add_health_item(scope, HealthSeverity::Warning, message, Some(reason.to_owned())).await;
    if let Err(error) = recorded {
        tracing::error!(%error, stream = stream.as_str(), "a refused stream's health item was not recorded");
    }
}

fn telemetry_failure(error: impl ToString) -> ProviderExecutionFailure {
    failure("sidecar_telemetry", error.to_string())
}

fn lease_failure(error: impl ToString) -> ProviderExecutionFailure {
    failure("sidecar_leases", error.to_string())
}

fn lease_id((_, lease): &(String, String)) -> String {
    lease.clone()
}

fn remove_matching_lease(held: &mut std::collections::BTreeMap<String, String>, token: &str, lease: &str) {
    if held.get(token).is_some_and(|current| current == lease) {
        held.remove(token);
    }
}

/* === Tests === */
