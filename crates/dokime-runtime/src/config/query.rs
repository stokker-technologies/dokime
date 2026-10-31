/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Immutable Finite Query Budget Composition

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports
use harmos::operations::OperationDeclaration;
use harmos::storage::StorageSettings;

// Local Imports
use dokime_domain::model::query::{
    error::{EffectiveLimit, Failure, Reason},
    page::{PageLimits, PageRequest},
};
use dokime_operations::query::{AggregateSamples, ListCycles, ListExecutions, ListRecords, ListSamples};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug)]
pub struct QuerySettings {
    pub page_rows: u32,
    pub page_bytes: u64,
    pub max_page_rows: u32,
    pub max_page_bytes: u64,
    pub working_bytes: u64,
    pub schema_bytes: u64,
    pub scanned_rows: u64,
    pub scanned_bytes: u64,
    pub active_time: Duration,
    pub admission_time: Duration,
    pub concurrency: usize,
    pub pending: usize,
    pub predicate_terms: usize,
    pub in_members: usize,
    pub fields: usize,
    pub versions: usize,
    pub alternatives: usize,
    pub partitions: usize,
    pub buckets: usize,
    pub aggregate_outputs: usize,
    pub token_bytes: usize,
    pub snapshot_lifetime: Duration,
}

// Only Composition Can Construct This Snapshot; Callers Can Borrow Its Limits.
#[derive(Clone, Debug)]
pub struct QueryConfig {
    limits: QuerySettings,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl QuerySettings {
    pub fn compose(&self, storage: &StorageSettings, schema: &harmos::config::SchemaSettings) -> Result<QueryConfig, Box<Failure>> {
        let positive = self.page_rows > 0
            && self.max_page_rows > 0
            && self.page_bytes > 0
            && self.max_page_bytes > 0
            && self.working_bytes > 0
            && self.schema_bytes > 0
            && self.scanned_rows > 0
            && self.scanned_bytes > 0
            && [
                self.concurrency,
                self.pending,
                self.predicate_terms,
                self.in_members,
                self.fields,
                self.versions,
                self.alternatives,
                self.partitions,
                self.buckets,
                self.aggregate_outputs,
                self.token_bytes,
            ]
            .iter()
            .all(|value| *value > 0)
            && !self.active_time.is_zero()
            && !self.admission_time.is_zero()
            && !self.snapshot_lifetime.is_zero();
        if !positive
            || self.page_rows > self.max_page_rows
            || self.page_bytes > self.max_page_bytes
            || self.admission_time > self.active_time
            || self.snapshot_lifetime.as_nanos() > u128::from(u64::MAX)
            || usize::try_from(self.working_bytes).is_err()
            || usize::try_from(self.max_page_bytes).is_err()
            || usize::try_from(self.schema_bytes).is_err()
        {
            return Err(Failure::new(
                Reason::InvalidSelection,
                "dokime.query.configure",
                "Query budgets must be positive, addressable and ordered within their ceilings",
            ));
        }
        for policy in [ListExecutions::POLICY, ListCycles::POLICY, ListRecords::POLICY, ListSamples::POLICY, AggregateSamples::POLICY] {
            if self.active_time > policy.timeout || self.active_time > policy.attempt_timeout {
                return Err(Failure::new(
                    Reason::InvalidSelection,
                    "dokime.query.configure",
                    "Query active-work deadline exceeds a declared operation ceiling",
                ));
            }
        }
        let mut limits = self.clone();
        limits.max_page_rows = limits.max_page_rows.min(u32::try_from(storage.response.batch_rows.get()).unwrap_or(u32::MAX));
        limits.max_page_bytes = limits.max_page_bytes.min(storage.response.batch_bytes.get() as u64);
        limits.working_bytes = limits.working_bytes.min(schema.preparation_bytes.get() as u64);
        limits.schema_bytes = limits.schema_bytes.min(limits.working_bytes).min(limits.max_page_bytes);
        limits.page_rows = limits.page_rows.min(limits.max_page_rows);
        limits.page_bytes = limits.page_bytes.min(limits.max_page_bytes);
        limits.concurrency = limits.concurrency.min(storage.request.max_active_queries.get());
        limits.pending = limits.pending.min(storage.request.max_pending_queries.get());
        limits.fields = limits.fields.min(schema.preparation_work.get());
        Ok(QueryConfig { limits })
    }
}

impl QueryConfig {
    pub fn limits(&self) -> &QuerySettings {
        &self.limits
    }

    // The Serving Transport May Impose A Smaller Message Ceiling On Any Call.
    pub fn page_limits(&self, request: &PageRequest, message_bytes: usize, operation: &str) -> Result<PageLimits, Box<Failure>> {
        if request.rows == Some(0) || request.bytes == Some(0) || message_bytes == 0 {
            return Err(Failure::new(Reason::InvalidSelection, operation, "Page and transport budgets must be positive"));
        }
        Ok(PageLimits {
            rows: request.rows.unwrap_or(self.limits.page_rows).min(self.limits.max_page_rows),
            bytes: request.bytes.unwrap_or(self.limits.page_bytes).min(self.limits.max_page_bytes).min(message_bytes as u64),
        })
    }

    pub fn bound(&self, used: usize, maximum: usize, name: &str, operation: &str) -> Result<(), Box<Failure>> {
        if used <= maximum {
            return Ok(());
        }
        let mut error =
            Failure::new(Reason::ResourceLimit, operation, "Query complexity exceeds the configured limit; narrow the selection");
        error.limit = Some(EffectiveLimit {
            name: name.to_owned(),
            value: maximum as u64,
        });
        Err(error)
    }
}

/* === Public Functions === */

pub fn query_defaults() -> QuerySettings {
    let storage = super::storage_defaults();
    let batch = storage.response.batch_bytes.get() as u64;
    QuerySettings {
        page_rows: 256,
        page_bytes: batch / 8,
        max_page_rows: storage.response.batch_rows.get() as u32,
        max_page_bytes: batch,
        working_bytes: batch,
        schema_bytes: batch / 8,
        scanned_rows: 4_194_304,
        scanned_bytes: batch * 128,
        active_time: ListSamples::POLICY.attempt_timeout / 2,
        admission_time: ListSamples::POLICY.attempt_timeout / 10,
        concurrency: storage.request.max_active_queries.get(),
        pending: storage.request.max_pending_queries.get(),
        predicate_terms: 64,
        in_members: 256,
        fields: 128,
        versions: 32,
        alternatives: 256,
        partitions: 64,
        buckets: 4096,
        aggregate_outputs: 64,
        token_bytes: 4096,
        snapshot_lifetime: Duration::from_secs(15 * 60),
    }
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composition_boundaries() {
        let mut storage = super::super::storage_defaults();
        let mut schema = super::super::schema_defaults();
        let settings = query_defaults();
        storage.response.batch_rows = std::num::NonZeroUsize::new(2).unwrap();
        storage.response.batch_bytes = std::num::NonZeroUsize::new(128).unwrap();
        storage.request.max_active_queries = std::num::NonZeroUsize::new(1).unwrap();
        schema.preparation_bytes = std::num::NonZeroUsize::new(64).unwrap();
        schema.preparation_work = std::num::NonZeroUsize::new(2).unwrap();
        let config = settings.compose(&storage, &schema).unwrap();
        assert_eq!(config.limits().working_bytes, 64);
        assert_eq!(config.limits().schema_bytes, 64);
        assert_eq!(config.limits().concurrency, 1);
        assert_eq!(config.limits().fields, 2);
        let request = PageRequest {
            rows: Some(100),
            bytes: Some(1000),
            after: None,
        };
        assert_eq!(config.page_limits(&request, 80, "test.query").unwrap(), PageLimits { rows: 2, bytes: 80 });
        assert_eq!(
            config.page_limits(&PageRequest { rows: None, bytes: None, after: None }, 1024, "test.query").unwrap(),
            PageLimits { rows: 2, bytes: 128 }
        );
        assert!(config.page_limits(&request, 0, "test.query").is_err());
        let mut invalid = settings.clone();
        invalid.active_time = ListSamples::POLICY.attempt_timeout + Duration::from_nanos(1);
        assert!(invalid.compose(&storage, &schema).is_err());
        invalid.active_time = ListSamples::POLICY.attempt_timeout;
        assert!(invalid.compose(&storage, &schema).is_ok());
        invalid.page_rows = invalid.max_page_rows + 1;
        assert!(invalid.compose(&storage, &schema).is_err());
        for invalid in [
            QuerySettings { page_rows: 0, ..settings.clone() },
            QuerySettings { page_bytes: 0, ..settings.clone() },
            QuerySettings { max_page_rows: 0, ..settings.clone() },
            QuerySettings { max_page_bytes: 0, ..settings.clone() },
            QuerySettings { working_bytes: 0, ..settings.clone() },
            QuerySettings { schema_bytes: 0, ..settings.clone() },
            QuerySettings { scanned_rows: 0, ..settings.clone() },
            QuerySettings { scanned_bytes: 0, ..settings.clone() },
            QuerySettings {
                active_time: Duration::ZERO,
                ..settings.clone()
            },
            QuerySettings {
                admission_time: Duration::ZERO,
                ..settings.clone()
            },
            QuerySettings {
                snapshot_lifetime: Duration::ZERO,
                ..settings.clone()
            },
            QuerySettings {
                snapshot_lifetime: Duration::MAX,
                ..settings.clone()
            },
            QuerySettings { concurrency: 0, ..settings.clone() },
            QuerySettings { pending: 0, ..settings.clone() },
            QuerySettings { predicate_terms: 0, ..settings.clone() },
            QuerySettings { in_members: 0, ..settings.clone() },
            QuerySettings { fields: 0, ..settings.clone() },
            QuerySettings { versions: 0, ..settings.clone() },
            QuerySettings { alternatives: 0, ..settings.clone() },
            QuerySettings { partitions: 0, ..settings.clone() },
            QuerySettings { buckets: 0, ..settings.clone() },
            QuerySettings { aggregate_outputs: 0, ..settings.clone() },
            QuerySettings { token_bytes: 0, ..settings.clone() },
        ] {
            assert!(invalid.compose(&storage, &schema).is_err());
        }
        let error = config.bound(3, 2, "fields", "test.query").unwrap_err();
        assert_eq!(error.limit.unwrap().value, 2);
        assert!(config.bound(2, 2, "fields", "test.query").is_ok());
    }
}
