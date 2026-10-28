/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;

use harmos::ExecutionContext;
use harmos::operations::{NoStream, OperationDeclaration, Requires};
use harmos::sidecar::operations::Carrier;
use harmos::sidecar::{Refusal, Supervisor};

use crate::records::DeclaredRecord;
use dokime_domain::model::records::evidence::Record;

pub const LISTENER_POLICY: harmos::operations::OperationPolicy = harmos::operations::OperationPolicy {
    timeout: std::time::Duration::from_secs(30),
    attempt_timeout: std::time::Duration::from_secs(30),
    retries: harmos::operations::RetryPolicy::Never,
};

pub type HearFuture<'a> = Pin<Box<dyn Future<Output = Result<(), Refusal>> + Send + 'a>>;

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct Delivery {
    #[row(description = "JSON-encoded [`Record`](dokime_domain::model::records::evidence::Record), as the host filed it.")]
    pub record_json: String,
}

// The carrier that serves one listener's callback operation: the host
// delivers the filed record under the listener's own operation id.
pub struct Hearing<L>(PhantomData<fn() -> L>);

// One delivery attempt as a listener sees it: the declared resources and the
// attempt's own host, which publishes and calls under this attempt.
pub struct ListenerContext<'a, L> {
    execution: &'a ExecutionContext,
    listener: PhantomData<fn() -> L>,
}

// A listener is a callback operation over `Delivery`; `#[dokime::listener]`
// declares its identity, resources and record type.
pub trait Listener: Sized + OperationDeclaration<Request = Delivery, Response = (), Input = NoStream, Output = NoStream> {
    type Record: DeclaredRecord;

    fn hear<'a>(ctx: &'a ListenerContext<'a, Self>, record: Self::Record) -> HearFuture<'a>;
}

impl<L> ListenerContext<'_, L> {
    pub fn call(&self) -> &harmos::operations::CallContext {
        &self.execution.call
    }

    // Registration verified every declared resource before this attempt began.
    pub fn resource<T: Send + Sync + 'static>(&self) -> Arc<T>
    where
        L: Requires<T>,
    {
        self.execution.resource::<T>().unwrap_or_else(|error| panic!("declared listener resource unavailable after verification: {error}"))
    }

    // The attempt's host: publications and calls travel under this delivery.
    pub fn host(&self) -> Arc<Supervisor>
    where
        L: Requires<Supervisor>,
    {
        self.resource::<Supervisor>()
    }
}

impl<L: Listener> Carrier for Hearing<L> {
    type Declaration = L;
    type Error = Refusal;
    type StreamEndpoints = ();

    async fn bind(_: ExecutionContext) -> Result<Self, Refusal> {
        Ok(Self(PhantomData))
    }

    async fn attempt(&mut self, context: ExecutionContext, request: &mut Delivery, _: &mut ()) -> Result<harmos::Response<()>, Refusal> {
        let record = request.record::<L::Record>().map_err(Refusal::of)?;
        let ctx = ListenerContext {
            execution: &context,
            listener: PhantomData,
        };
        L::hear(&ctx, record).await?;
        Ok(().into())
    }
}

impl Delivery {
    pub fn of(record: &Record) -> Self {
        Self {
            record_json: serde_json::to_string(record).expect("a filed record serializes"),
        }
    }

    pub fn record<R: DeclaredRecord>(&self) -> crate::Result<R> {
        let record: Record = serde_json::from_str(&self.record_json).map_err(|error| crate::Error::RecordDecode {
            record: R::RECORD_TYPE.to_owned(),
            reason: error.to_string(),
        })?;
        R::from_record(&record)
    }
}
