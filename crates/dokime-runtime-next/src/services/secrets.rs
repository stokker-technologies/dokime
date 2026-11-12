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
impl v1::secrets_server::Secrets for Services {
    async fn export(&self, request: tonic::Request<v1::SecretsExportRequest>) -> Result<tonic::Response<v1::SecretArchive>, tonic::Status> {
        let reply = dokime_operations_next::client::secrets_export::handle(&self.context, request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunExportStream = dokime_operations_next::client::Output<v1::SecretsExportRunOutput>;
    async fn run_export(&self, request: tonic::Request<tonic::Streaming<v1::SecretsExportRunInput>>) -> Result<tonic::Response<Self::RunExportStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(self.context.clone(), request.into_inner(), self.context.settings.read_timeout,
            |frame| frame.value.map(|value| match value {
                v1::secrets_export_run_input::Value::Request(request) => RunFrame::Request(request),
                v1::secrets_export_run_input::Value::Finish(_) => RunFrame::Finish,
                v1::secrets_export_run_input::Value::Cancel(_) => RunFrame::Cancel,
            }),
            |reply| v1::SecretsExportRunOutput { value: Some(v1::secrets_export_run_output::Value::Response(reply)) },
            |context, request| async move { dokime_operations_next::client::secrets_export::handle(&context, request).await },
        ).await
    }
    async fn import(&self, request: tonic::Request<v1::SecretsImportRequest>) -> Result<tonic::Response<v1::SecretSnapshot>, tonic::Status> {
        let reply = dokime_operations_next::client::secrets_import::handle(&self.context, request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunImportStream = dokime_operations_next::client::Output<v1::SecretsImportRunOutput>;
    async fn run_import(&self, request: tonic::Request<tonic::Streaming<v1::SecretsImportRunInput>>) -> Result<tonic::Response<Self::RunImportStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(self.context.clone(), request.into_inner(), self.context.settings.mutation_timeout,
            |frame| frame.value.map(|value| match value {
                v1::secrets_import_run_input::Value::Request(request) => RunFrame::Request(request),
                v1::secrets_import_run_input::Value::Finish(_) => RunFrame::Finish,
                v1::secrets_import_run_input::Value::Cancel(_) => RunFrame::Cancel,
            }),
            |reply| v1::SecretsImportRunOutput { value: Some(v1::secrets_import_run_output::Value::Response(reply)) },
            |context, request| async move { dokime_operations_next::client::secrets_import::handle(&context, request).await },
        ).await
    }
    async fn list(&self, request: tonic::Request<v1::SecretsListRequest>) -> Result<tonic::Response<v1::SecretSnapshot>, tonic::Status> {
        let reply = dokime_operations_next::client::secrets_list::handle(&self.context, request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListStream = dokime_operations_next::client::Output<v1::SecretsListRunOutput>;
    async fn run_list(&self, request: tonic::Request<tonic::Streaming<v1::SecretsListRunInput>>) -> Result<tonic::Response<Self::RunListStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(self.context.clone(), request.into_inner(), self.context.settings.read_timeout,
            |frame| frame.value.map(|value| match value {
                v1::secrets_list_run_input::Value::Request(request) => RunFrame::Request(request),
                v1::secrets_list_run_input::Value::Finish(_) => RunFrame::Finish,
                v1::secrets_list_run_input::Value::Cancel(_) => RunFrame::Cancel,
            }),
            |reply| v1::SecretsListRunOutput { value: Some(v1::secrets_list_run_output::Value::Response(reply)) },
            |context, request| async move { dokime_operations_next::client::secrets_list::handle(&context, request).await },
        ).await
    }
    async fn lookup(&self, request: tonic::Request<v1::SecretsLookupRequest>) -> Result<tonic::Response<v1::SecretLookupValue>, tonic::Status> {
        let reply = dokime_operations_next::client::secrets_lookup::handle(&self.context, request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunLookupStream = dokime_operations_next::client::Output<v1::SecretsLookupRunOutput>;
    async fn run_lookup(&self, request: tonic::Request<tonic::Streaming<v1::SecretsLookupRunInput>>) -> Result<tonic::Response<Self::RunLookupStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(self.context.clone(), request.into_inner(), self.context.settings.read_timeout,
            |frame| frame.value.map(|value| match value {
                v1::secrets_lookup_run_input::Value::Request(request) => RunFrame::Request(request),
                v1::secrets_lookup_run_input::Value::Finish(_) => RunFrame::Finish,
                v1::secrets_lookup_run_input::Value::Cancel(_) => RunFrame::Cancel,
            }),
            |reply| v1::SecretsLookupRunOutput { value: Some(v1::secrets_lookup_run_output::Value::Response(reply)) },
            |context, request| async move { dokime_operations_next::client::secrets_lookup::handle(&context, request).await },
        ).await
    }
    async fn preview(&self, request: tonic::Request<v1::SecretsPreviewRequest>) -> Result<tonic::Response<v1::SecretImportPreview>, tonic::Status> {
        let reply = dokime_operations_next::client::secrets_preview::handle(&self.context, request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunPreviewStream = dokime_operations_next::client::Output<v1::SecretsPreviewRunOutput>;
    async fn run_preview(&self, request: tonic::Request<tonic::Streaming<v1::SecretsPreviewRunInput>>) -> Result<tonic::Response<Self::RunPreviewStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(self.context.clone(), request.into_inner(), self.context.settings.read_timeout,
            |frame| frame.value.map(|value| match value {
                v1::secrets_preview_run_input::Value::Request(request) => RunFrame::Request(request),
                v1::secrets_preview_run_input::Value::Finish(_) => RunFrame::Finish,
                v1::secrets_preview_run_input::Value::Cancel(_) => RunFrame::Cancel,
            }),
            |reply| v1::SecretsPreviewRunOutput { value: Some(v1::secrets_preview_run_output::Value::Response(reply)) },
            |context, request| async move { dokime_operations_next::client::secrets_preview::handle(&context, request).await },
        ).await
    }
    async fn put(&self, request: tonic::Request<v1::SecretsPutRequest>) -> Result<tonic::Response<v1::SecretSnapshot>, tonic::Status> {
        let reply = dokime_operations_next::client::secrets_put::handle(&self.context, request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunPutStream = dokime_operations_next::client::Output<v1::SecretsPutRunOutput>;
    async fn run_put(&self, request: tonic::Request<tonic::Streaming<v1::SecretsPutRunInput>>) -> Result<tonic::Response<Self::RunPutStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(self.context.clone(), request.into_inner(), self.context.settings.mutation_timeout,
            |frame| frame.value.map(|value| match value {
                v1::secrets_put_run_input::Value::Request(request) => RunFrame::Request(request),
                v1::secrets_put_run_input::Value::Finish(_) => RunFrame::Finish,
                v1::secrets_put_run_input::Value::Cancel(_) => RunFrame::Cancel,
            }),
            |reply| v1::SecretsPutRunOutput { value: Some(v1::secrets_put_run_output::Value::Response(reply)) },
            |context, request| async move { dokime_operations_next::client::secrets_put::handle(&context, request).await },
        ).await
    }
    async fn remove(&self, request: tonic::Request<v1::SecretsRemoveRequest>) -> Result<tonic::Response<v1::SecretSnapshot>, tonic::Status> {
        let reply = dokime_operations_next::client::secrets_remove::handle(&self.context, request.into_inner()).await.map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunRemoveStream = dokime_operations_next::client::Output<v1::SecretsRemoveRunOutput>;
    async fn run_remove(&self, request: tonic::Request<tonic::Streaming<v1::SecretsRemoveRunInput>>) -> Result<tonic::Response<Self::RunRemoveStream>, tonic::Status> {
        use dokime_operations_next::client::{RunFrame, run_unary};
        run_unary(self.context.clone(), request.into_inner(), self.context.settings.mutation_timeout,
            |frame| frame.value.map(|value| match value {
                v1::secrets_remove_run_input::Value::Request(request) => RunFrame::Request(request),
                v1::secrets_remove_run_input::Value::Finish(_) => RunFrame::Finish,
                v1::secrets_remove_run_input::Value::Cancel(_) => RunFrame::Cancel,
            }),
            |reply| v1::SecretsRemoveRunOutput { value: Some(v1::secrets_remove_run_output::Value::Response(reply)) },
            |context, request| async move { dokime_operations_next::client::secrets_remove::handle(&context, request).await },
        ).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
