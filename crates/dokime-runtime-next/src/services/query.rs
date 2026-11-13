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
impl v1::query_server::Query for Services {
    async fn aggregate(
        &self, request: tonic::Request<v1::QueryAggregateRequest>,
    ) -> Result<tonic::Response<v1::AggregateReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Query.Aggregate: Not Yet Served by the New Host"))
    }
    type RunAggregateStream = dokime_operations_next::client::Output<v1::QueryAggregateRunOutput>;
    async fn run_aggregate(
        &self, request: tonic::Request<tonic::Streaming<v1::QueryAggregateRunInput>>,
    ) -> Result<tonic::Response<Self::RunAggregateStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Query.RunAggregate: Not Yet Served by the New Host"))
    }
    async fn aggregate_samples(
        &self, request: tonic::Request<v1::QueryAggregateSamplesRequest>,
    ) -> Result<tonic::Response<v1::Statistics>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Query.AggregateSamples: Not Yet Served by the New Host"))
    }
    type RunAggregateSamplesStream = dokime_operations_next::client::Output<v1::QueryAggregateSamplesRunOutput>;
    async fn run_aggregate_samples(
        &self, request: tonic::Request<tonic::Streaming<v1::QueryAggregateSamplesRunInput>>,
    ) -> Result<tonic::Response<Self::RunAggregateSamplesStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Query.RunAggregateSamples: Not Yet Served by the New Host"))
    }
    async fn list_cycles(
        &self, request: tonic::Request<v1::QueryListCyclesRequest>,
    ) -> Result<tonic::Response<v1::CyclePage>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Query.ListCycles: Not Yet Served by the New Host"))
    }
    type RunListCyclesStream = dokime_operations_next::client::Output<v1::QueryListCyclesRunOutput>;
    async fn run_list_cycles(
        &self, request: tonic::Request<tonic::Streaming<v1::QueryListCyclesRunInput>>,
    ) -> Result<tonic::Response<Self::RunListCyclesStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Query.RunListCycles: Not Yet Served by the New Host"))
    }
    async fn list_executions(
        &self, request: tonic::Request<v1::QueryListExecutionsRequest>,
    ) -> Result<tonic::Response<v1::ExecutionPage>, tonic::Status> {
        let reply = dokime_operations_next::client::query_list_executions::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListExecutionsStream = dokime_operations_next::client::Output<v1::QueryListExecutionsRunOutput>;
    async fn run_list_executions(
        &self, request: tonic::Request<tonic::Streaming<v1::QueryListExecutionsRunInput>>,
    ) -> Result<tonic::Response<Self::RunListExecutionsStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Query.RunListExecutions: Not Yet Served by the New Host"))
    }
    async fn list_records(
        &self, request: tonic::Request<v1::QueryListRecordsRequest>,
    ) -> Result<tonic::Response<v1::RecordPage>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Query.ListRecords: Not Yet Served by the New Host"))
    }
    type RunListRecordsStream = dokime_operations_next::client::Output<v1::QueryListRecordsRunOutput>;
    async fn run_list_records(
        &self, request: tonic::Request<tonic::Streaming<v1::QueryListRecordsRunInput>>,
    ) -> Result<tonic::Response<Self::RunListRecordsStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Query.RunListRecords: Not Yet Served by the New Host"))
    }
    async fn list_samples(
        &self, request: tonic::Request<v1::QueryListSamplesRequest>,
    ) -> Result<tonic::Response<v1::SamplePage>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Query.ListSamples: Not Yet Served by the New Host"))
    }
    type RunListSamplesStream = dokime_operations_next::client::Output<v1::QueryListSamplesRunOutput>;
    async fn run_list_samples(
        &self, request: tonic::Request<tonic::Streaming<v1::QueryListSamplesRunInput>>,
    ) -> Result<tonic::Response<Self::RunListSamplesStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Query.RunListSamples: Not Yet Served by the New Host"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
