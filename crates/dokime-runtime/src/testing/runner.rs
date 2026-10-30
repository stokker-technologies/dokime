/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Adverse Runner Fixtures Still Enter Through Typed SDK Operations And The
// Supervisor Wire, Including Their Evidence, Signals And Cancellation.

use std::{marker::PhantomData, sync::Arc};

use crate::execution::{Runnable, SampleOutput, Served};
use harmos::{
    operations::{NoStream, OperationDeclaration, Requires},
    sidecar::{Refusal, Supervisor},
};

pub struct Runner<R>(pub Arc<R>);
pub struct Run<R>(pub crate::transport::RunExecution, PhantomData<fn() -> R>);
pub struct Signal<R>(pub crate::transport::ExecutionSignal, PhantomData<fn() -> R>);

crate::transport::templated!(Run, crate::transport::RunExecution);
crate::transport::templated!(Signal, crate::transport::ExecutionSignal);

impl<R: Runnable + Served> OperationDeclaration for Run<R> {
    type Request = Self;
    type Response = crate::transport::SidecarExecutionOutcome;
    type Input = NoStream;
    type Output = NoStream;
    const ID: &'static str = R::RUN;
    const DESCRIPTION: &'static str = "Runs one case of this template to its outcome.";
    const POLICY: harmos::operations::OperationPolicy = <crate::execution::Run<R> as OperationDeclaration>::POLICY;
    const VISIBILITY: harmos::operations::Visibility = harmos::operations::Visibility::Host;
}

impl<R: Runnable + Served> OperationDeclaration for Signal<R> {
    type Request = Self;
    type Response = crate::transport::SignalReply;
    type Input = NoStream;
    type Output = NoStream;
    const ID: &'static str = R::SIGNAL;
    const DESCRIPTION: &'static str = "Reaches one running case of this template.";
    const POLICY: harmos::operations::OperationPolicy = R::SIGNAL_POLICY;
    const VISIBILITY: harmos::operations::Visibility = harmos::operations::Visibility::Host;
}

impl<R: Runnable + Served> Requires<Runner<R>> for Run<R> {}
impl<R: Runnable + Served> Requires<Supervisor> for Run<R> {}
impl<R: Runnable + Served> Requires<Runner<R>> for Signal<R> {}

impl<R: Runnable + Served> harmos::Dependent for Run<R> {
    const RESOURCES: &'static [harmos::Resource] = &[harmos::Resource::of::<Runner<R>>(), harmos::Resource::of::<Supervisor>()];
}
impl<R: Runnable + Served> harmos::Dependent for Signal<R> {
    const RESOURCES: &'static [harmos::Resource] = &[harmos::Resource::of::<Runner<R>>()];
}
impl<R: Runnable + Served> harmos::Operation for Run<R> {
    type Error = Refusal;
    async fn run(self, ctx: harmos::Context<'_, Self>) -> Result<harmos::Response<Self::Response>, Refusal> {
        let host = Supervisor::clone(&ctx.resource::<Supervisor>());
        ctx.resource::<Runner<R>>()
            .0
            .run(self.0, Arc::new(host.clone()), Some(SampleOutput::Host(host.clone())), crate::execution::Caller::host(host))
            .await
            .map(Into::into)
    }
}
impl<R: Runnable + Served> harmos::Operation for Signal<R> {
    type Error = Refusal;
    async fn run(self, ctx: harmos::Context<'_, Self>) -> Result<harmos::Response<Self::Response>, Refusal> {
        ctx.resource::<Runner<R>>().0.signal(self.0).await.map(Into::into)
    }
}
