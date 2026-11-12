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
impl v1::catalog_server::Catalog for Services {
    async fn get_record_schema(
        &self, request: tonic::Request<v1::CatalogGetRecordSchemaRequest>,
    ) -> Result<tonic::Response<v1::RecordSchema>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Catalog.GetRecordSchema: Not Yet Served by the New Host"))
    }
    type RunGetRecordSchemaStream = dokime_operations_next::client::Output<v1::CatalogGetRecordSchemaRunOutput>;
    async fn run_get_record_schema(
        &self, request: tonic::Request<tonic::Streaming<v1::CatalogGetRecordSchemaRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetRecordSchemaStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Catalog.RunGetRecordSchema: Not Yet Served by the New Host"))
    }
    async fn get_suite(&self, request: tonic::Request<v1::CatalogGetSuiteRequest>) -> Result<tonic::Response<v1::Suite>, tonic::Status> {
        let reply = dokime_operations_next::client::catalog_get_suite::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunGetSuiteStream = dokime_operations_next::client::Output<v1::CatalogGetSuiteRunOutput>;
    async fn run_get_suite(
        &self, request: tonic::Request<tonic::Streaming<v1::CatalogGetSuiteRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetSuiteStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Catalog.RunGetSuite: Not Yet Served by the New Host"))
    }
    async fn get_telemetry_schema(
        &self, request: tonic::Request<v1::CatalogGetTelemetrySchemaRequest>,
    ) -> Result<tonic::Response<v1::TelemetrySchema>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Catalog.GetTelemetrySchema: Not Yet Served by the New Host"))
    }
    type RunGetTelemetrySchemaStream = dokime_operations_next::client::Output<v1::CatalogGetTelemetrySchemaRunOutput>;
    async fn run_get_telemetry_schema(
        &self, request: tonic::Request<tonic::Streaming<v1::CatalogGetTelemetrySchemaRunInput>>,
    ) -> Result<tonic::Response<Self::RunGetTelemetrySchemaStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Catalog.RunGetTelemetrySchema: Not Yet Served by the New Host"))
    }
    async fn list_record_schemas(
        &self, request: tonic::Request<v1::CatalogListRecordSchemasRequest>,
    ) -> Result<tonic::Response<v1::ListRecordSchemasReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Catalog.ListRecordSchemas: Not Yet Served by the New Host"))
    }
    type RunListRecordSchemasStream = dokime_operations_next::client::Output<v1::CatalogListRecordSchemasRunOutput>;
    async fn run_list_record_schemas(
        &self, request: tonic::Request<tonic::Streaming<v1::CatalogListRecordSchemasRunInput>>,
    ) -> Result<tonic::Response<Self::RunListRecordSchemasStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Catalog.RunListRecordSchemas: Not Yet Served by the New Host"))
    }
    async fn list_suites(
        &self, request: tonic::Request<v1::CatalogListSuitesRequest>,
    ) -> Result<tonic::Response<v1::ListSuitesReply>, tonic::Status> {
        let reply = dokime_operations_next::client::catalog_list_suites::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListSuitesStream = dokime_operations_next::client::Output<v1::CatalogListSuitesRunOutput>;
    async fn run_list_suites(
        &self, request: tonic::Request<tonic::Streaming<v1::CatalogListSuitesRunInput>>,
    ) -> Result<tonic::Response<Self::RunListSuitesStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Catalog.RunListSuites: Not Yet Served by the New Host"))
    }
    async fn list_telemetry_schemas(
        &self, request: tonic::Request<v1::CatalogListTelemetrySchemasRequest>,
    ) -> Result<tonic::Response<v1::ListTelemetrySchemasReply>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Catalog.ListTelemetrySchemas: Not Yet Served by the New Host"))
    }
    type RunListTelemetrySchemasStream = dokime_operations_next::client::Output<v1::CatalogListTelemetrySchemasRunOutput>;
    async fn run_list_telemetry_schemas(
        &self, request: tonic::Request<tonic::Streaming<v1::CatalogListTelemetrySchemasRunInput>>,
    ) -> Result<tonic::Response<Self::RunListTelemetrySchemasStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Catalog.RunListTelemetrySchemas: Not Yet Served by the New Host"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
