/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl v1::backend_server::Backend for Services {
    type CollectDiagnosticsStream = dokime_operations_next::client::Output<v1::BackendCollectDiagnosticsOutput>;
    async fn collect_diagnostics(
        &self, request: tonic::Request<v1::CollectBackendDiagnostics>,
    ) -> Result<tonic::Response<Self::CollectDiagnosticsStream>, tonic::Status> {
        let output = dokime_operations_next::client::backend_collect_diagnostics::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(output))
    }
    type RunCollectDiagnosticsStream = dokime_operations_next::client::Output<v1::BackendCollectDiagnosticsRunOutput>;
    async fn run_collect_diagnostics(
        &self, request: tonic::Request<tonic::Streaming<v1::BackendCollectDiagnosticsRunInput>>,
    ) -> Result<tonic::Response<Self::RunCollectDiagnosticsStream>, tonic::Status> {
        let mut input = request.into_inner();
        let first = tokio::time::timeout(self.context.settings.read_timeout, input.message())
            .await
            .map_err(|_| tonic::Status::deadline_exceeded("Run Diagnostics: Send the Request Before Its Deadline"))??;
        let Some(v1::backend_collect_diagnostics_run_input::Value::Request(request)) = first.and_then(|frame| frame.value) else {
            return Err(tonic::Status::invalid_argument("Run Diagnostics: Send One Request First"));
        };
        let mut output = dokime_operations_next::client::backend_collect_diagnostics::handle(&self.context, request)
            .await
            .map_err(dokime_operations_next::client::status)?;
        let (sender, receiver) = tokio::sync::mpsc::channel(self.context.settings.stream_buffer);
        tokio::spawn(async move {
            use tokio_stream::StreamExt;
            let mut finished = false;
            loop {
                tokio::select! {
                    _ = sender.closed() => return,
                    frame = input.message(), if !finished => match frame {
                        Ok(Some(frame)) => match frame.value {
                            Some(v1::backend_collect_diagnostics_run_input::Value::Finish(_)) => finished = true,
                            Some(v1::backend_collect_diagnostics_run_input::Value::Cancel(_)) => { let _ = sender.send(Err(tonic::Status::cancelled("Run Diagnostics: Cancelled"))).await; return; }
                            _ => { let _ = sender.send(Err(tonic::Status::invalid_argument("Run Diagnostics: Send Only Finish or Cancel After the Request"))).await; return; }
                        },
                        Ok(None) => finished = true,
                        Err(status) => { let _ = sender.send(Err(status)).await; return; }
                    },
                    frame = output.next() => {
                        let Some(frame) = frame else { return; };
                        let frame = frame.map(|frame| v1::BackendCollectDiagnosticsRunOutput { value: frame.value.map(|value| match value {
                            v1::backend_collect_diagnostics_output::Value::Item(item) => v1::backend_collect_diagnostics_run_output::Value::Item(item),
                            v1::backend_collect_diagnostics_output::Value::Response(response) => v1::backend_collect_diagnostics_run_output::Value::Response(response),
                        }) });
                        if sender.send(frame).await.is_err() { return; }
                    }
                }
            }
        });
        Ok(tonic::Response::new(dokime_operations_next::client::Output::new(receiver)))
    }
    async fn get_availability(
        &self, request: tonic::Request<v1::GetBackendAvailability>,
    ) -> Result<tonic::Response<v1::BackendAvailability>, tonic::Status> {
        let reply = dokime_operations_next::client::backend_get_availability::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetAvailabilityStream = dokime_operations_next::client::Output<v1::BackendGetAvailabilityRunOutput>;
    async fn run_get_availability(
        &self, request: tonic::Request<tonic::Streaming<v1::BackendGetAvailabilityRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetAvailabilityStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(
            self.context.clone(),
            request.into_inner(),
            self.context.settings.read_timeout,
            |frame| {
                frame.value.map(|value| match value {
                    v1::backend_get_availability_run_input::Value::Request(request) => RunFrame::Request(request),
                    v1::backend_get_availability_run_input::Value::Finish(_) => RunFrame::Finish,
                    v1::backend_get_availability_run_input::Value::Cancel(_) => RunFrame::Cancel,
                })
            },
            |reply| v1::BackendGetAvailabilityRunOutput {
                value: Some(v1::backend_get_availability_run_output::Value::Response(reply)),
            },
            |context, request| async move { dokime_operations_next::client::backend_get_availability::handle(&context, request).await },
        )
        .await
    }
    async fn get_health(&self, request: tonic::Request<v1::GetBackendHealth>) -> Result<tonic::Response<v1::BackendHealth>, tonic::Status> {
        let reply = dokime_operations_next::client::backend_get_health::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetHealthStream = dokime_operations_next::client::Output<v1::BackendGetHealthRunOutput>;
    async fn run_get_health(
        &self, request: tonic::Request<tonic::Streaming<v1::BackendGetHealthRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetHealthStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(
            self.context.clone(),
            request.into_inner(),
            self.context.settings.read_timeout,
            |frame| {
                frame.value.map(|value| match value {
                    v1::backend_get_health_run_input::Value::Request(request) => RunFrame::Request(request),
                    v1::backend_get_health_run_input::Value::Finish(_) => RunFrame::Finish,
                    v1::backend_get_health_run_input::Value::Cancel(_) => RunFrame::Cancel,
                })
            },
            |reply| v1::BackendGetHealthRunOutput {
                value: Some(v1::backend_get_health_run_output::Value::Response(reply)),
            },
            |context, request| async move { dokime_operations_next::client::backend_get_health::handle(&context, request).await },
        )
        .await
    }
    async fn get_identity(
        &self, request: tonic::Request<v1::GetBackendIdentity>,
    ) -> Result<tonic::Response<v1::BackendIdentity>, tonic::Status> {
        let reply = dokime_operations_next::client::backend_get_identity::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetIdentityStream = dokime_operations_next::client::Output<v1::BackendGetIdentityRunOutput>;
    async fn run_get_identity(
        &self, request: tonic::Request<tonic::Streaming<v1::BackendGetIdentityRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetIdentityStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(
            self.context.clone(),
            request.into_inner(),
            self.context.settings.read_timeout,
            |frame| {
                frame.value.map(|value| match value {
                    v1::backend_get_identity_run_input::Value::Request(request) => RunFrame::Request(request),
                    v1::backend_get_identity_run_input::Value::Finish(_) => RunFrame::Finish,
                    v1::backend_get_identity_run_input::Value::Cancel(_) => RunFrame::Cancel,
                })
            },
            |reply| v1::BackendGetIdentityRunOutput {
                value: Some(v1::backend_get_identity_run_output::Value::Response(reply)),
            },
            |context, request| async move { dokime_operations_next::client::backend_get_identity::handle(&context, request).await },
        )
        .await
    }
    type WatchLogsStream = dokime_operations_next::client::Output<v1::BackendWatchLogsOutput>;
    async fn watch_logs(
        &self, request: tonic::Request<v1::WatchBackendLogs>,
    ) -> Result<tonic::Response<Self::WatchLogsStream>, tonic::Status> {
        let output = dokime_operations_next::client::backend_watch_logs::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(output))
    }
    type RunWatchLogsStream = dokime_operations_next::client::Output<v1::BackendWatchLogsRunOutput>;
    async fn run_watch_logs(
        &self, request: tonic::Request<tonic::Streaming<v1::BackendWatchLogsRunInput>>,
    ) -> Result<tonic::Response<Self::RunWatchLogsStream>, tonic::Status> {
        let mut input = request.into_inner();
        let first = tokio::time::timeout(self.context.settings.read_timeout, input.message())
            .await
            .map_err(|_| tonic::Status::deadline_exceeded("Run Diagnostics: Send the Request Before Its Deadline"))??;
        let Some(v1::backend_watch_logs_run_input::Value::Request(request)) = first.and_then(|frame| frame.value) else {
            return Err(tonic::Status::invalid_argument("Run Diagnostics: Send One Request First"));
        };
        let stop_on_finish = true;
        let mut output = dokime_operations_next::client::backend_watch_logs::handle(&self.context, request)
            .await
            .map_err(dokime_operations_next::client::status)?;
        let (sender, receiver) = tokio::sync::mpsc::channel(self.context.settings.stream_buffer);
        tokio::spawn(async move {
            use tokio_stream::StreamExt;
            let mut finished = false;
            loop {
                tokio::select! {
                    _ = sender.closed() => return,
                    frame = input.message(), if !finished => match frame {
                        Ok(Some(frame)) => match frame.value {
                            Some(v1::backend_watch_logs_run_input::Value::Finish(_)) => {
                                if stop_on_finish { let _ = sender.send(Ok(v1::BackendWatchLogsRunOutput { value: Some(v1::backend_watch_logs_run_output::Value::Response(v1::BackendWatchLogsCompletion {})) })).await; return; }
                                finished = true;
                            },
                            Some(v1::backend_watch_logs_run_input::Value::Cancel(_)) => { let _ = sender.send(Err(tonic::Status::cancelled("Run Diagnostics: Cancelled"))).await; return; }
                            _ => { let _ = sender.send(Err(tonic::Status::invalid_argument("Run Diagnostics: Send Only Finish or Cancel After the Request"))).await; return; }
                        },
                        Ok(None) => finished = true,
                        Err(status) => { let _ = sender.send(Err(status)).await; return; }
                    },
                    frame = output.next() => {
                        let Some(frame) = frame else { return; };
                        let frame = frame.map(|frame| v1::BackendWatchLogsRunOutput { value: frame.value.map(|value| match value {
                            v1::backend_watch_logs_output::Value::Item(item) => v1::backend_watch_logs_run_output::Value::Item(item),
                            v1::backend_watch_logs_output::Value::Response(response) => v1::backend_watch_logs_run_output::Value::Response(response),
                        }) });
                        if sender.send(frame).await.is_err() { return; }
                    }
                }
            }
        });
        Ok(tonic::Response::new(dokime_operations_next::client::Output::new(receiver)))
    }
    type WatchTraceStream = dokime_operations_next::client::Output<v1::BackendWatchTraceOutput>;
    async fn watch_trace(
        &self, request: tonic::Request<v1::WatchBackendTrace>,
    ) -> Result<tonic::Response<Self::WatchTraceStream>, tonic::Status> {
        let output = dokime_operations_next::client::backend_watch_trace::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(output))
    }
    type RunWatchTraceStream = dokime_operations_next::client::Output<v1::BackendWatchTraceRunOutput>;
    async fn run_watch_trace(
        &self, request: tonic::Request<tonic::Streaming<v1::BackendWatchTraceRunInput>>,
    ) -> Result<tonic::Response<Self::RunWatchTraceStream>, tonic::Status> {
        let mut input = request.into_inner();
        let first = tokio::time::timeout(self.context.settings.read_timeout, input.message())
            .await
            .map_err(|_| tonic::Status::deadline_exceeded("Run Diagnostics: Send the Request Before Its Deadline"))??;
        let Some(v1::backend_watch_trace_run_input::Value::Request(request)) = first.and_then(|frame| frame.value) else {
            return Err(tonic::Status::invalid_argument("Run Diagnostics: Send One Request First"));
        };
        let stop_on_finish = request.follow;
        let mut output = dokime_operations_next::client::backend_watch_trace::handle(&self.context, request)
            .await
            .map_err(dokime_operations_next::client::status)?;
        let (sender, receiver) = tokio::sync::mpsc::channel(self.context.settings.stream_buffer);
        tokio::spawn(async move {
            use tokio_stream::StreamExt;
            let mut finished = false;
            loop {
                tokio::select! {
                    _ = sender.closed() => return,
                    frame = input.message(), if !finished => match frame {
                        Ok(Some(frame)) => match frame.value {
                            Some(v1::backend_watch_trace_run_input::Value::Finish(_)) => {
                                if stop_on_finish { let _ = sender.send(Ok(v1::BackendWatchTraceRunOutput { value: Some(v1::backend_watch_trace_run_output::Value::Response(v1::BackendWatchTraceCompletion {})) })).await; return; }
                                finished = true;
                            },
                            Some(v1::backend_watch_trace_run_input::Value::Cancel(_)) => { let _ = sender.send(Err(tonic::Status::cancelled("Run Diagnostics: Cancelled"))).await; return; }
                            _ => { let _ = sender.send(Err(tonic::Status::invalid_argument("Run Diagnostics: Send Only Finish or Cancel After the Request"))).await; return; }
                        },
                        Ok(None) => finished = true,
                        Err(status) => { let _ = sender.send(Err(status)).await; return; }
                    },
                    frame = output.next() => {
                        let Some(frame) = frame else { return; };
                        let frame = frame.map(|frame| v1::BackendWatchTraceRunOutput { value: frame.value.map(|value| match value {
                            v1::backend_watch_trace_output::Value::Item(item) => v1::backend_watch_trace_run_output::Value::Item(item),
                            v1::backend_watch_trace_output::Value::Response(response) => v1::backend_watch_trace_run_output::Value::Response(response),
                        }) });
                        if sender.send(frame).await.is_err() { return; }
                    }
                }
            }
        });
        Ok(tonic::Response::new(dokime_operations_next::client::Output::new(receiver)))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
