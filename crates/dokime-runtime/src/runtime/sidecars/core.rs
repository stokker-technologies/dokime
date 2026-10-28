/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::sync::{Arc, RwLock};

use harmos::operations::HostContext;
use harmos::{Context, RegistrationExt};
use tonic::Status;

use crate::RuntimeHandle;
use crate::app::OperationManager;
use crate::operations::hosting::{Host, Serve, Served};
use crate::records::RecordPublication;

#[derive(Default)]
pub(crate) struct RecordIngress {
    runtime: RwLock<Option<RuntimeHandle>>,
}

impl RecordIngress {
    pub(crate) async fn register(operations: &OperationManager) -> harmos::Result<Arc<Self>> {
        let ingress = Arc::new(Self::default());
        let mut dependencies = harmos::Dependencies::new();
        dependencies.insert(Host::<RecordPublication>::new(ingress.clone()))?;
        let batch = harmos::RegistrationBatch::new(dependencies).operation::<RecordPublication>();
        operations.register(batch.lane(crate::app::Lanes::Open)).await?;
        Ok(ingress)
    }

    pub(crate) fn attach(&self, runtime: RuntimeHandle) {
        *self.runtime.write().expect("record ingress is healthy") = Some(runtime);
    }

    pub(crate) fn detach(&self) {
        self.runtime.write().expect("record ingress is healthy").take();
    }

    fn runtime(&self) -> Result<RuntimeHandle, Status> {
        self.runtime.read().expect("record ingress is healthy").clone().ok_or_else(|| Status::unavailable("record ingress is closing"))
    }
}

impl Serve<RecordPublication> for RecordIngress {
    fn serve<'a>(&'a self, publication: RecordPublication, ctx: Context<'a, RecordPublication>) -> Served<'a, ()> {
        Box::pin(async move {
            let owner = ctx
                .resource::<HostContext>()
                .borrow()
                .caller_scope()
                .map(|scope| scope.owner().to_owned())
                .ok_or_else(|| Status::invalid_argument("the original plugin caller is required"))?;
            let runtime = self.runtime()?;
            let Some((request, produced_at)) = super::record_publication(&owner, publication) else {
                return Err(Status::invalid_argument("the publication is not a record publish request"));
            };
            let record_type = request.record_type.clone();
            runtime.publish_sidecar_record_at(request, &owner, produced_at).await.map(|_| ().into()).map_err(|rejection| {
                tracing::warn!(
                    owner,
                    record_type = record_type.as_str(),
                    detail = ?rejection,
                    "refusing a record publication"
                );
                Status::invalid_argument("the record was refused against the plugin's declared contracts")
            })
        })
    }
}

// Serves Harmos's `streams.publish` built-in; each call reaches the calling
// sidecar's own native endpoint through the call's host context.
pub(crate) async fn register_publications(operations: &OperationManager) -> harmos::Result<()> {
    let batch = harmos::RegistrationBatch::new(harmos::Dependencies::new()).operation::<harmos::builtins::streams::Publish>();
    operations.register(batch.lane(crate::app::Lanes::Open)).await
}
