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

// Third Party Imports

// Local Imports
use super::{Client, RequestStream, ResponseStream, Result, contracts};
use harmos::MemorySize;
use harmos::domain::Portable;
use harmos::operations::Direction;
use harmos::operations::OperationDeclaration;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Client {
    /// Calls any unary endpoint; the request type selects the operation.
    pub async fn call<O>(&self, request: O) -> Result<O::Response>
    where
        O: OperationDeclaration<Request = O> + Sync,
        O::Response: Portable + MemorySize,
    {
        super::transport::value(self.unary::<O>(request).await?)
    }

    /// Opens any server-streaming endpoint; the request type selects the operation.
    pub async fn stream<O>(&self, request: O) -> Result<ResponseStream<<O::Output as Direction>::Item>>
    where
        O: OperationDeclaration<Request = O> + Sync,
        O::Response: Portable + MemorySize,
        <O::Output as Direction>::Item: Portable + MemorySize,
    {
        self.download::<O>(request).await
    }

    pub async fn content_upload_artifact(
        &self, message: impl tokio_stream::Stream<Item = dokime_domain::model::content::wire::UploadArtifactFrame> + Send + 'static,
    ) -> Result<dokime_domain::model::content::wire::ArtifactMetadata> {
        self.upload::<contracts::content::UploadArtifact, _>(contracts::content::UploadArtifact {}, message).await
    }

    pub async fn plugins_install(
        &self, message: impl tokio_stream::Stream<Item = dokime_domain::model::plugins::wire::InstallPluginFrame> + Send + 'static,
    ) -> Result<dokime_domain::model::plugins::wire::PluginInstallation> {
        self.upload::<contracts::plugins::Install, _>(contracts::plugins::Install {}, message).await
    }

    pub async fn reports_download_cycle_report(
        &self, message: contracts::reports::DownloadCycleReport,
    ) -> Result<ResponseStream<dokime_domain::model::reports::wire::ReportHtmlFrame>> {
        self.with_timeout(std::time::Duration::from_millis(300_000)).download::<contracts::reports::DownloadCycleReport>(message).await
    }

    pub async fn reports_download_execution_report(
        &self, message: contracts::reports::DownloadExecutionReport,
    ) -> Result<ResponseStream<dokime_domain::model::reports::wire::ReportHtmlFrame>> {
        self.with_timeout(std::time::Duration::from_millis(300_000)).download::<contracts::reports::DownloadExecutionReport>(message).await
    }

    pub fn content_upload_artifact_open(
        &self,
    ) -> Result<
        RequestStream<dokime_domain::model::content::wire::UploadArtifactFrame, dokime_domain::model::content::wire::ArtifactMetadata>,
    > {
        self.require(&dokime_operations::path::<contracts::content::UploadArtifact>())?;
        let client = self.clone();
        RequestStream::new(self, harmos::grpc::prepare_input_item::<contracts::content::UploadArtifact>, move |stream| async move {
            client.upload_prepared::<contracts::content::UploadArtifact, _>(contracts::content::UploadArtifact {}, stream).await
        })
    }

    pub fn plugins_install_open(
        &self,
    ) -> Result<
        RequestStream<dokime_domain::model::plugins::wire::InstallPluginFrame, dokime_domain::model::plugins::wire::PluginInstallation>,
    > {
        self.require(&dokime_operations::path::<contracts::plugins::Install>())?;
        let client = self.clone();
        RequestStream::new(self, harmos::grpc::prepare_input_item::<contracts::plugins::Install>, move |stream| async move {
            client.upload_prepared::<contracts::plugins::Install, _>(contracts::plugins::Install {}, stream).await
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
