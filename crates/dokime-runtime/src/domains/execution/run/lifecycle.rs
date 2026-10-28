/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Settle Execution Publications, Cancellation And Capture Leases.

use super::{
    ExecutionCleanup, ExecutionFixtureContext, ExecutionSignal, Finalization, HostCapture, Identifier, Instant, ProgressCleanup,
    ProviderExecutionFailure, StepLeaseBridge, TestExecution, close_record_subscriptions, failure, sampling_scope,
};

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
        let mut active = self.active.lock().await;
        if let Some(cleanup) = active.as_mut() {
            cleanup.finish(context).await?;
        }
        *active = None;
        Ok(())
    }
}

impl ExecutionCleanup {
    pub(super) async fn finish(&mut self, context: &ExecutionFixtureContext) -> Result<(), ProviderExecutionFailure> {
        self.settle(context).await;

        let publication_started = Instant::now();
        tracing::debug!(execution_id = %self.execution.test_execution_id, "execution publication flush started");
        context.runtime.flush_plugin_publications().await.map_err(telemetry_failure)?;
        tracing::debug!(execution_id = %self.execution.test_execution_id,
            elapsed_ms = publication_started.elapsed().as_millis() as u64, "execution publication flush settled");
        close_record_subscriptions(context, &self.execution).await?;
        self.session.prompts.lock().await.settle_on_cancel(context).await;

        let capture_started = Instant::now();
        tracing::debug!(execution_id = %self.execution.test_execution_id, "execution capture release started");
        self.session.step_leases.release_all(context).await?;
        self.capture.release(context).await?;
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
        match context.runtime.lease_streams(sampling_scope(execution), streams, source).await {
            Ok(leases) => Self { leases },
            Err(error) => {
                let description = {
                    let names = streams.iter().map(Identifier::to_string).collect::<Vec<_>>();
                    names.join(", ")
                };
                tracing::warn!(
                    plugin_id = source,
                    streams = description,
                    %error,
                    "declared streams could not be leased"
                );
                Self::default()
            }
        }
    }

    pub(super) async fn release(&mut self, context: &ExecutionFixtureContext) -> Result<(), ProviderExecutionFailure> {
        context.runtime.release_stream_leases(self.leases.clone()).await.map_err(lease_failure)?;
        self.leases.clear();
        Ok(())
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
