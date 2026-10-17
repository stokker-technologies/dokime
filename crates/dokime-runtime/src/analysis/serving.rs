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
use std::sync::Arc;

// Third Party Imports
use harmos::sidecar::{Refusal, Supervisor};

// Local Imports
use super::{Analysis, Error, Operation, Request, Response};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct Handler<A> {
    state: State,
    analysis: std::marker::PhantomData<fn() -> A>,
}

pub struct Served<A, R = ()>(Handler<A>, std::marker::PhantomData<fn() -> R>);

struct State {
    consumed: bool,
    worker: Option<tokio::task::JoinHandle<Result<Response, Error>>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl<A: Analysis> harmos::operations::Operation for Handler<A> {
    type Declaration = Operation<A>;
    type Resources = ();
    type Error = Refusal;
    type Context<'a> = harmos::operations::Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(_: Arc<()>, _: Self::Context<'ctx>) -> Result<Self, Refusal> {
        Ok(Self::fresh())
    }
    async fn run<'ctx>(
        &mut self,
        _: Self::Context<'ctx>,
        request: &mut Request,
        _: &mut (),
    ) -> Result<harmos::operations::Response<Response>, Refusal> {
        self.state.evaluate::<A>(request).await.map(Into::into)
    }
    async fn stop(&mut self) -> Result<(), Refusal> {
        self.state.settle().await
    }
}

impl<A: Analysis, R> harmos::operations::Declares for Served<A, R> {
    type Declaration = Operation<A>;
}

impl<A: Analysis, R: Send + Sync + 'static> harmos::operations::Operation for Served<A, R> {
    type Declaration = Operation<A>;
    type Resources = R;
    type Error = Refusal;
    type Context<'a> = &'a Supervisor;
    type Endpoints = ();

    async fn initialize<'ctx>(_: Arc<R>, _: Self::Context<'ctx>) -> Result<Self, Refusal> {
        Ok(Self(Handler::fresh(), std::marker::PhantomData))
    }
    async fn run<'ctx>(
        &mut self,
        _: Self::Context<'ctx>,
        request: &mut Request,
        _: &mut (),
    ) -> Result<harmos::operations::Response<Response>, Refusal> {
        self.0.state.evaluate::<A>(request).await.map(Into::into)
    }
    async fn stop(&mut self) -> Result<(), Refusal> {
        self.0.state.settle().await
    }
}

impl<A: Analysis> Handler<A> {
    fn fresh() -> Self {
        Self {
            state: State {
                consumed: false,
                worker: None,
            },
            analysis: std::marker::PhantomData,
        }
    }
}

impl State {
    fn check<A: Analysis>(request: &Request) -> Result<(), Refusal> {
        if request.parameters.len() > super::MAX_PARAMETER_BYTES {
            return Err(Refusal::of("analysis configuration exceeds its budget"));
        }
        request
            .samples
            .as_ref()
            .ok_or_else(|| Refusal::of("analysis samples are missing"))?
            .check(A::LIMITS)
            .map_err(Refusal::of)?;
        Ok(())
    }

    async fn evaluate<A: Analysis>(&mut self, request: &mut Request) -> Result<Response, Refusal> {
        if self.worker.is_none() {
            if self.consumed {
                return Err(Refusal::of("analysis attempt was already consumed"));
            }
            Self::check::<A>(request)?;
            let request = std::mem::take(request);
            self.consumed = true;
            self.worker = Some(tokio::task::spawn_blocking(move || {
                Operation::<A>::evaluate(request)
            }));
        }
        let result = self
            .worker
            .as_mut()
            .expect("retained analysis worker")
            .await;
        self.worker.take();
        result.map_err(Refusal::of)?.map_err(Refusal::of)
    }
    async fn settle(&mut self) -> Result<(), Refusal> {
        // Borrow Rather Than Take: Cancelled Finalization Resumes The Same Worker.
        if let Some(worker) = self.worker.as_mut() {
            let result = worker.await;
            self.worker.take();
            let _ = result.map_err(Refusal::of)?;
        }
        self.consumed = true;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
