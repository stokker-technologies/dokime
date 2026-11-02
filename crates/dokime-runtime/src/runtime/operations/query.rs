/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Serve Finite Read Queries

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{future::Future, sync::Arc};

// Third Party Imports
use harmos::{Context, Operation};
use tokio::sync::{Semaphore, SemaphorePermit};
use tonic::{Code, Status};
use tonic_types::{ErrorDetails, StatusExt};

// Local Imports
use crate::app::{Dokime, Lanes, LiveRuntime, live_runtime};
use crate::config::QueryConfig;
use crate::operations::hosting::{Host, Serve, Served};
use crate::operations::query::{AggregateSamples, ListCycles, ListExecutions, ListRecords, ListSamples};
use crate::runtime::query::Caller;
use dokime_domain::model::query::{
    error::{Failure, Reason, Status as FailureStatus},
    page::{CyclePage, ExecutionPage, RecordPage, SamplePage},
    reduction::Statistics,
};

/* === Constants === */

/* === Macros === */

macro_rules! serve {
    ($request:ty, $response:ty, $method:ident) => {
        impl Serve<$request> for Queries {
            fn serve<'a>(&'a self, request: $request, ctx: Context<'a, $request>) -> Served<'a, $response> {
                Box::pin(async move {
                    let caller = ctx.call().get::<Caller>();
                    let maximum = caller.map_or(self.config.limits().max_page_bytes as usize, |caller| caller.message_bytes);
                    self.run::<$request, _>(&ctx, async {
                        let runtime = live_runtime(&self.live).await?;
                        runtime.$method(request, maximum).await.map(Into::into).map_err(|mut failure| {
                            failure.operation = <$request as harmos::OperationDeclaration>::ID.into();
                            status(failure)
                        })
                    })
                    .await
                })
            }
        }
    };
}

/* === Type Aliases === */

/* === Structs === */

struct Queries {
    live: LiveRuntime,
    config: QueryConfig,
    active: Semaphore,
    pending: Semaphore,
}

/* === Enums === */

#[harmos::catalog]
enum Operations {
    Executions(ListExecutions),
    Cycles(ListCycles),
    Records(ListRecords),
    Samples(ListSamples),
    Statistics(AggregateSamples),
}

/* === Traits === */

/* === Implementations === */

serve!(ListExecutions, ExecutionPage, query_list_executions);
serve!(ListCycles, CyclePage, query_list_cycles);
serve!(ListRecords, RecordPage, query_list_records);
serve!(ListSamples, SamplePage, query_list_samples);
serve!(AggregateSamples, Statistics, query_aggregate_samples);

impl Queries {
    async fn admit(&self, operation: &str) -> Result<SemaphorePermit<'_>, Status> {
        if let Ok(permit) = self.active.try_acquire() {
            return Ok(permit);
        }
        let _waiting =
            self.pending.try_acquire().map_err(|_| status(Failure::new(Reason::ResourceLimit, operation, "Query admission is full")))?;
        tokio::time::timeout(self.config.limits().admission_time, self.active.acquire())
            .await
            .map_err(|_| status(Failure::new(Reason::ResourceLimit, operation, "Query admission wait exceeded its configured limit")))?
            .map_err(|_| status(Failure::new(Reason::StorageUnavailable, operation, "Query admission is closed")))
    }

    async fn run<O: Operation, T>(&self, ctx: &Context<'_, O>, work: impl Future<Output = Result<T, Status>>) -> Result<T, Status> {
        let revoked = ctx.call().get::<Caller>().and_then(|caller| caller.revoked.as_ref());
        self.read(O::ID, ctx.cancelled(), revoked, work).await
    }

    async fn read<T>(
        &self, operation: &str, cancelled: impl Future<Output = ()>, revoked: Option<&tokio_util::sync::CancellationToken>,
        work: impl Future<Output = Result<T, Status>>,
    ) -> Result<T, Status> {
        let revocation = async {
            match revoked {
                Some(revoked) => revoked.cancelled().await,
                None => std::future::pending().await,
            }
        };
        tokio::select! {
            biased;
            () = cancelled => Err(Status::cancelled("Query cancelled")),
            () = revocation => Err(status(Failure::new(Reason::PermissionDenied, operation, "Query access was revoked"))),
            result = async {
                let _permit = self.admit(operation).await?;
                work.await
            } => result,
        }
    }
}

/* === Public Functions === */

pub(crate) fn prepare(setup: &mut harmos::Setup<Dokime>, live: &LiveRuntime, config: &QueryConfig) {
    let served = Arc::new(Queries {
        live: live.clone(),
        config: config.clone(),
        active: Semaphore::new(config.limits().concurrency),
        pending: Semaphore::new(config.limits().pending),
    });
    setup.resource::<Host<ListExecutions>>(Host::new(served.clone()));
    setup.resource::<Host<ListCycles>>(Host::new(served.clone()));
    setup.resource::<Host<ListRecords>>(Host::new(served.clone()));
    setup.resource::<Host<ListSamples>>(Host::new(served.clone()));
    setup.resource::<Host<AggregateSamples>>(Host::new(served));
    setup.operations::<Operations>(Lanes::Open);
}

/* === Private Functions === */

fn status(mut failure: Box<Failure>) -> Status {
    use dokime_domain::model::diagnostics::redacted;
    failure.message = redacted(&failure.message);
    failure.field = failure.field.map(|field| redacted(&field));
    let code = match failure.status() {
        FailureStatus::InvalidArgument => Code::InvalidArgument,
        FailureStatus::PermissionDenied => Code::PermissionDenied,
        FailureStatus::NotFound => Code::NotFound,
        FailureStatus::FailedPrecondition => Code::FailedPrecondition,
        FailureStatus::ResourceExhausted => Code::ResourceExhausted,
        FailureStatus::DeadlineExceeded => Code::DeadlineExceeded,
        FailureStatus::DataLoss => Code::DataLoss,
        FailureStatus::Unavailable => Code::Unavailable,
    };
    let reason = serde_json::to_value(failure.reason).expect("query failure reasons serialize as strings");
    let reason = reason.as_str().expect("query failure reasons are strings").to_ascii_uppercase();
    let metadata =
        std::collections::HashMap::from([("query".into(), serde_json::to_string(&failure).expect("query failures have JSON-safe fields"))]);
    let details = ErrorDetails::with_error_info(reason, "dokime.query", metadata);
    Status::with_error_details(code, failure.message.clone(), details)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use harmos::OperationDeclaration;
    use std::sync::atomic::{AtomicBool, Ordering};

    fn queries() -> Queries {
        let settings = crate::config::query_defaults();
        let config = settings.compose(&crate::config::storage_defaults(), &crate::config::schema_defaults()).unwrap();
        Queries {
            live: Default::default(),
            config,
            active: Semaphore::new(1),
            pending: Semaphore::new(1),
        }
    }

    #[tokio::test(start_paused = true)]
    async fn admission_bounds_waiters_and_releases_cancelled_capacity() {
        let queries = queries();
        let active = queries.admit(ListCycles::ID).await.unwrap();
        let mut waiting = Box::pin(queries.admit(ListCycles::ID));
        assert!(futures_util::poll!(&mut waiting).is_pending());
        assert_eq!(queries.admit(ListSamples::ID).await.unwrap_err().code(), Code::ResourceExhausted);
        drop(waiting);
        let expired = queries.admit(ListSamples::ID).await.unwrap_err();
        assert_eq!(expired.code(), Code::ResourceExhausted);
        drop(active);
        assert!(queries.admit(ListCycles::ID).await.is_ok());
    }

    #[tokio::test]
    async fn cancellation_and_revocation_drop_active_work_and_release_admission() {
        struct Dropped(Arc<AtomicBool>);
        impl Drop for Dropped {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }
        for revoke in [false, true] {
            let queries = queries();
            let dropped = Arc::new(AtomicBool::new(false));
            let guard = Dropped(dropped.clone());
            let work = async move {
                let _guard = guard;
                std::future::pending::<Result<(), Status>>().await
            };
            let cancelled = tokio_util::sync::CancellationToken::new();
            let revoked = tokio_util::sync::CancellationToken::new();
            let mut pending = Box::pin(queries.read(ListSamples::ID, cancelled.cancelled(), Some(&revoked), work));
            assert!(futures_util::poll!(&mut pending).is_pending());
            assert!(queries.active.try_acquire().is_err());
            if revoke {
                revoked.cancel();
            } else {
                cancelled.cancel();
            }
            let error = pending.await.unwrap_err();
            assert_eq!(error.code(), if revoke { Code::PermissionDenied } else { Code::Cancelled });
            assert!(dropped.load(Ordering::SeqCst));
            assert!(queries.admit(ListCycles::ID).await.is_ok());
        }
    }

    #[test]
    fn failure_mapping_preserves_typed_details_and_redacts_paths() {
        let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap();
        for (reason, code) in [
            (Reason::UnknownField, Code::InvalidArgument),
            (Reason::PermissionDenied, Code::PermissionDenied),
            (Reason::ScopeNotFound, Code::NotFound),
            (Reason::SnapshotUnavailable, Code::FailedPrecondition),
            (Reason::WorkLimit, Code::ResourceExhausted),
            (Reason::Deadline, Code::DeadlineExceeded),
            (Reason::Corruption, Code::DataLoss),
            (Reason::StorageUnavailable, Code::Unavailable),
        ] {
            let mut failure = Failure::new(reason, ListSamples::ID, format!("Cannot read {home}/evidence"));
            failure.field = Some(format!("{home}/field"));
            failure.limit = Some(dokime_domain::model::query::error::EffectiveLimit { name: "rows".into(), value: 2 });
            let error = status(failure);
            assert_eq!(error.code(), code);
            assert!(!error.message().contains(&home));
            let details = error.get_error_details();
            let info = details.error_info().unwrap();
            assert_eq!(info.domain, "dokime.query");
            let decoded: Failure = serde_json::from_str(&info.metadata["query"]).unwrap();
            assert_eq!(decoded.reason, reason);
            assert_eq!(decoded.operation, ListSamples::ID);
            assert_eq!(decoded.limit.unwrap().value, 2);
            assert!(!decoded.field.unwrap().contains(&home));
        }
    }
}
