/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plots Client Service

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
impl v1::plots_server::Plots for Services {
    async fn list_analyses(
        &self, request: tonic::Request<v1::PlotsListAnalysesRequest>,
    ) -> Result<tonic::Response<v1::ListAnalysesReply>, tonic::Status> {
        let reply = dokime_operations_next::client::plots_list_analyses::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListAnalysesStream = dokime_operations_next::client::Output<v1::PlotsListAnalysesRunOutput>;
    async fn run_list_analyses(
        &self, request: tonic::Request<tonic::Streaming<v1::PlotsListAnalysesRunInput>>,
    ) -> Result<tonic::Response<Self::RunListAnalysesStream>, tonic::Status> {
        let reply = dokime_operations_next::client::run_call::unary(
            &self.context,
            request.into_inner(),
            |input| {
                input.value.map(|value| match value {
                    v1::plots_list_analyses_run_input::Value::Request(request) => {
                        dokime_operations_next::client::run_call::Input::Request(request)
                    }
                    v1::plots_list_analyses_run_input::Value::Cancel(_) => dokime_operations_next::client::run_call::Input::Cancel,
                    v1::plots_list_analyses_run_input::Value::Finish(_) => dokime_operations_next::client::run_call::Input::Finish,
                })
            },
            |context, request| async move { dokime_operations_next::client::plots_list_analyses::handle(&context, request).await },
            |reply| v1::PlotsListAnalysesRunOutput {
                value: Some(v1::plots_list_analyses_run_output::Value::Response(reply)),
            },
        )
        .await
        .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    async fn query_snapshots(
        &self, request: tonic::Request<v1::PlotsQuerySnapshotsRequest>,
    ) -> Result<tonic::Response<v1::PlotSnapshotsReply>, tonic::Status> {
        let reply = dokime_operations_next::client::plots_query_snapshots::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunQuerySnapshotsStream = dokime_operations_next::client::Output<v1::PlotsQuerySnapshotsRunOutput>;
    async fn run_query_snapshots(
        &self, request: tonic::Request<tonic::Streaming<v1::PlotsQuerySnapshotsRunInput>>,
    ) -> Result<tonic::Response<Self::RunQuerySnapshotsStream>, tonic::Status> {
        let reply = dokime_operations_next::client::run_call::unary(
            &self.context,
            request.into_inner(),
            |input| {
                input.value.map(|value| match value {
                    v1::plots_query_snapshots_run_input::Value::Request(request) => {
                        dokime_operations_next::client::run_call::Input::Request(request)
                    }
                    v1::plots_query_snapshots_run_input::Value::Cancel(_) => dokime_operations_next::client::run_call::Input::Cancel,
                    v1::plots_query_snapshots_run_input::Value::Finish(_) => dokime_operations_next::client::run_call::Input::Finish,
                })
            },
            |context, request| async move { dokime_operations_next::client::plots_query_snapshots::handle(&context, request).await },
            |reply| v1::PlotsQuerySnapshotsRunOutput {
                value: Some(v1::plots_query_snapshots_run_output::Value::Response(reply)),
            },
        )
        .await
        .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type WatchStream = dokime_operations_next::client::Output<v1::PlotsWatchOutput>;
    async fn watch(&self, request: tonic::Request<v1::PlotsWatchRequest>) -> Result<tonic::Response<Self::WatchStream>, tonic::Status> {
        let reply = dokime_operations_next::client::plots_watch::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunWatchStream = dokime_operations_next::client::Output<v1::PlotsWatchRunOutput>;
    async fn run_watch(
        &self, request: tonic::Request<tonic::Streaming<v1::PlotsWatchRunInput>>,
    ) -> Result<tonic::Response<Self::RunWatchStream>, tonic::Status> {
        let reply = dokime_operations_next::client::run_call::streaming(
            &self.context,
            request.into_inner(),
            |input| {
                input.value.map(|value| match value {
                    v1::plots_watch_run_input::Value::Request(request) => dokime_operations_next::client::run_call::Input::Request(request),
                    v1::plots_watch_run_input::Value::Cancel(_) => dokime_operations_next::client::run_call::Input::Cancel,
                    v1::plots_watch_run_input::Value::Finish(_) => dokime_operations_next::client::run_call::Input::Finish,
                })
            },
            |context, request| async move { dokime_operations_next::client::plots_watch::handle(&context, request).await },
            |frame| v1::PlotsWatchRunOutput {
                value: frame.value.map(|value| match value {
                    v1::plots_watch_output::Value::Item(item) => v1::plots_watch_run_output::Value::Item(item),
                    v1::plots_watch_output::Value::Response(response) => v1::plots_watch_run_output::Value::Response(response),
                }),
            },
            || v1::PlotsWatchRunOutput {
                value: Some(v1::plots_watch_run_output::Value::Response(v1::ClientEmpty {})),
            },
        )
        .await
        .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
