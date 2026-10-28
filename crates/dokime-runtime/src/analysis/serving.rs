/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use harmos::ExecutionContext;
use harmos::sidecar::Refusal;
use harmos::sidecar::operations::Carrier;

use super::{Analysis, Error, Operation, Request, Response};

// One analysis invocation: the request is consumed by exactly one blocking
// worker, which a cancelled attempt keeps and a retry or `stop` awaits; the
// evaluation never replays.
pub struct Evaluation<A> {
    consumed: bool,
    worker: Option<tokio::task::JoinHandle<Result<Response, Error>>>,
    analysis: std::marker::PhantomData<fn() -> A>,
}

impl<A: Analysis> harmos::Executable for Operation<A> {
    type Carrier = Evaluation<A>;
}

impl<A: Analysis> Carrier for Evaluation<A> {
    type Declaration = Operation<A>;
    type Error = Refusal;
    type StreamEndpoints = ();

    async fn bind(_: ExecutionContext) -> Result<Self, Refusal> {
        Ok(Self {
            consumed: false,
            worker: None,
            analysis: std::marker::PhantomData,
        })
    }

    async fn attempt(
        &mut self, _: ExecutionContext, request: &mut Operation<A>, _: &mut (),
    ) -> Result<harmos::Response<Response>, Refusal> {
        if self.worker.is_none() {
            if self.consumed {
                return Err(Refusal::of("analysis attempt was already consumed"));
            }
            check::<A>(&request.0)?;
            let request = std::mem::take(&mut request.0);
            self.consumed = true;
            self.worker = Some(tokio::task::spawn_blocking(move || Operation::<A>::evaluate(request)));
        }
        // Borrow rather than take: a cancelled attempt leaves the same worker for the next one.
        let result = self.worker.as_mut().expect("retained analysis worker").await;
        self.worker.take();
        result.map_err(Refusal::of)?.map(Into::into).map_err(Refusal::of)
    }

    async fn stop(&mut self) -> Result<(), Refusal> {
        // Borrow rather than take: cancelled finalization resumes the same worker.
        if let Some(worker) = self.worker.as_mut() {
            let result = worker.await;
            self.worker.take();
            let _ = result.map_err(Refusal::of)?;
        }
        self.consumed = true;
        Ok(())
    }
}

fn check<A: Analysis>(request: &Request) -> Result<(), Refusal> {
    if request.parameters.len() as u64 > dokime_domain::model::analysis::MAX_PARAMETER_BYTES as u64 {
        return Err(Refusal::of("analysis configuration exceeds its budget"));
    }
    request
        .samples
        .as_ref()
        .ok_or_else(|| Refusal::of("analysis samples are missing"))?
        .check(A::LIMITS.constrained())
        .map_err(Refusal::of)?;
    Ok(())
}
