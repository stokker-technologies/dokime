/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Fixed-Cut Typed Evidence Queries

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::HashSet, marker::PhantomData};

// Third Party Imports
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, MessageDescriptor};
use prost_types::Timestamp;

// Local Imports
use super::{EvidenceRecordsListOptions, EvidenceSamplesListOptions, Execution, ExecutionsGetOptions, FrozenSchemaCache, sample_row};
use crate::{Error, EvidenceScope, PageRequest, Payload, ReadCut, RecordScope, Result, payload, record_scope};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct RecordRow<T> {
    pub identifier: String,
    pub ordinal: u64,
    pub timestamp: Option<Timestamp>,
    pub value: T,
}

pub struct SampleRow<T> {
    pub identifier: String,
    pub ordinal: u64,
    pub timestamp: Option<Timestamp>,
    pub received_at: Option<Timestamp>,
    pub stream: String,
    pub source: String,
    pub value: T,
}

pub struct Records<T: Payload> {
    execution: Execution,
    identifier: String,
    step: String,
    marker: PhantomData<T>,
}

pub struct Samples<T: Payload> {
    execution: Execution,
    identifier: String,
    marker: PhantomData<T>,
}

struct Decoder<T: Payload> {
    descriptor: MessageDescriptor,
    marker: PhantomData<T>,
}

// One Query Owns Its First Durable Cut and Every Continuation at That Cut.
#[derive(Default)]
pub(super) struct Paging {
    pub at: Option<ReadCut>,
    pub token: String,
    seen: HashSet<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Execution {
    pub fn records<T: Payload>(&self, identifier: impl Into<String>) -> Records<T> {
        Records {
            execution: self.clone(),
            identifier: identifier.into(),
            step: String::new(),
            marker: PhantomData,
        }
    }

    pub fn samples<T: Payload>(&self, identifier: impl Into<String>) -> Samples<T> {
        Samples {
            execution: self.clone(),
            identifier: identifier.into(),
            marker: PhantomData,
        }
    }
}

impl Paging {
    pub fn advance(&mut self, at: Option<ReadCut>, token: String) -> Result<bool> {
        if at.is_none() || (self.at.is_some() && self.at != at) {
            return Err(Error::Invalid("Read Page: Preserve the First Durable Cut".into()));
        }
        self.at = at;
        if token.is_empty() {
            return Ok(false);
        }
        if !self.seen.insert(token.clone()) {
            return Err(Error::Invalid("Read Page: Return a New Continuation Token".into()));
        }
        self.token = token;
        Ok(true)
    }
}

impl<T: Payload> Records<T> {
    pub fn step(mut self, step: impl Into<String>) -> Self {
        self.step = step.into();
        self
    }

    pub async fn all(&self) -> Result<Vec<T>> {
        Ok(self.rows().await?.into_iter().map(|row| row.value).collect())
    }

    pub async fn count(&self) -> Result<u64> {
        let client = &self.execution.client;
        client.ensure_open()?;
        let page = client
            .resources
            .evidence
            .records
            .list_with(
                &self.identifier,
                EvidenceRecordsListOptions {
                    scope: Some(RecordScope {
                        kind: Some(record_scope::Kind::Execution(self.execution.identifier().into())),
                    }),
                    step: Some(self.step.clone()),
                    page: Some(PageRequest { page_size: 1, ..Default::default() }),
                    context: client.context.clone(),
                    ..Default::default()
                },
            )
            .await?;
        Ok(page.count)
    }

    pub async fn last(&self) -> Result<T> {
        let client = &self.execution.client;
        client.ensure_open()?;
        let page = client
            .resources
            .evidence
            .records
            .list_with(
                &self.identifier,
                EvidenceRecordsListOptions {
                    scope: Some(RecordScope {
                        kind: Some(record_scope::Kind::Execution(self.execution.identifier().into())),
                    }),
                    step: Some(self.step.clone()),
                    page: Some(PageRequest { page_size: 1, ..Default::default() }),
                    context: client.context.clone(),
                    ..Default::default()
                },
            )
            .await?;
        let record = page.last.ok_or_else(|| Error::Invalid("Read Record: Capture a Value Before Reading It".into()))?;
        Decoder::<T>::new(&self.execution, &self.identifier).await?.decode(&record.payload)
    }

    pub async fn rows(&self) -> Result<Vec<RecordRow<T>>> {
        let client = &self.execution.client;
        client.ensure_open()?;
        let mut paging = Paging::default();
        let mut rows = Vec::new();
        let decoder = Decoder::<T>::new(&self.execution, &self.identifier).await?;
        loop {
            let page = client
                .resources
                .evidence
                .records
                .list_with(
                    &self.identifier,
                    EvidenceRecordsListOptions {
                        scope: Some(RecordScope {
                            kind: Some(record_scope::Kind::Execution(self.execution.identifier().into())),
                        }),
                        step: Some(self.step.clone()),
                        at: paging.at.clone(),
                        page: Some(PageRequest {
                            page_token: paging.token.clone(),
                            ..Default::default()
                        }),
                        context: client.context.clone(),
                        ..Default::default()
                    },
                )
                .await?;
            for record in page.records {
                let value = decoder.decode(&record.payload)?;
                rows.push(RecordRow {
                    identifier: record.record_id,
                    ordinal: record.sequence,
                    timestamp: record.timestamp,
                    value,
                });
            }
            if !paging.advance(page.resolved_at, page.next_page_token)? {
                break;
            }
        }
        Ok(rows)
    }
}

impl<T: Payload> Samples<T> {
    pub async fn all(&self) -> Result<Vec<T>> {
        Ok(self.rows().await?.into_iter().map(|row| row.value).collect())
    }

    pub async fn rows(&self) -> Result<Vec<SampleRow<T>>> {
        let client = &self.execution.client;
        client.ensure_open()?;
        let mut paging = Paging::default();
        let mut rows = Vec::new();
        let decoder = Decoder::<T>::new(&self.execution, &self.identifier).await?;
        loop {
            let page = client
                .resources
                .evidence
                .samples
                .list_with(
                    EvidenceScope {
                        execution_id: Some(self.execution.identifier().into()),
                        ..Default::default()
                    },
                    vec![self.identifier.clone()],
                    EvidenceSamplesListOptions {
                        at: paging.at.clone(),
                        page: Some(PageRequest {
                            page_token: paging.token.clone(),
                            ..Default::default()
                        }),
                        context: client.context.clone(),
                        ..Default::default()
                    },
                )
                .await?;
            for row in page.samples {
                let Some(sample_row::Kind::Raw(sample)) = row.kind else {
                    return Err(Error::Invalid("Read Samples: Return Raw Rows for an Unbucketed Query".into()));
                };
                let value = decoder.decode(&sample.payload)?;
                rows.push(SampleRow {
                    identifier: sample.sample_id,
                    ordinal: sample.sequence,
                    timestamp: sample.timestamp,
                    received_at: sample.received_at,
                    stream: sample.stream,
                    source: sample.source,
                    value,
                });
            }
            if !paging.advance(page.resolved_at, page.next_page_token)? {
                break;
            }
        }
        Ok(rows)
    }
}

impl<T: Payload> Decoder<T> {
    async fn new(execution: &Execution, identifier: &str) -> Result<Self> {
        let client = &execution.client;
        let page = client
            .resources
            .executions
            .get_with(
                execution.identifier(),
                ExecutionsGetOptions {
                    context: client.context.clone(),
                    ..Default::default()
                },
            )
            .await?;
        let detail = page.execution.ok_or_else(|| Error::Invalid("Decode Evidence: Return the Frozen Execution".into()))?;
        let mut schemas = FrozenSchemaCache::default();
        let types = schemas.get(client, &detail).await?;
        let bindings = detail.schemas.as_ref().ok_or_else(|| Error::Invalid("Decode Evidence: Return Frozen Schema Bindings".into()))?;
        let name = bindings
            .records
            .iter()
            .find(|record| record.record_type == identifier)
            .map(|record| record.message_name.as_str())
            .or_else(|| bindings.streams.iter().find(|stream| stream.stream == identifier).map(|stream| stream.message_name.as_str()))
            .ok_or_else(|| Error::Invalid("Decode Evidence: Select a Frozen Record or Stream Identity".into()))?;
        let expected = <T::Message as harmos::Message>::name();
        if name != expected {
            return Err(Error::Invalid(format!("Decode Evidence: Select the Frozen Row Type {name}, Rather Than {expected}")));
        }
        let pool = DescriptorPool::from_file_descriptor_set(types.as_ref().clone())
            .map_err(|error| Error::Invalid(format!("Read Frozen Schemas: {error}")))?;
        let descriptor = pool
            .get_message_by_name(name)
            .ok_or_else(|| Error::Invalid("Decode Evidence: Restore the Frozen Message Descriptor".into()))?;
        Ok(Self { descriptor, marker: PhantomData })
    }

    fn decode(&self, bytes: &[u8]) -> Result<T> {
        let frozen = DynamicMessage::decode(self.descriptor.clone(), bytes)
            .map_err(|error| Error::Invalid(format!("Decode Frozen Evidence: {error}")))?;
        payload::decode(&frozen.encode_to_vec())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::Paging;
    use crate::{Position, ReadCut};

    #[test]
    fn pages_retain_the_first_cut_and_refuse_repeated_tokens() {
        let cut = ReadCut {
            positions: vec![Position { sequence: 9, ..Default::default() }],
            ..Default::default()
        };
        let mut paging = Paging::default();
        assert!(paging.advance(Some(cut.clone()), "second".into()).unwrap());
        assert!(paging.advance(Some(cut.clone()), "second".into()).is_err());
        assert!(paging.advance(Some(ReadCut::default()), "third".into()).is_err());
        assert!(paging.advance(None, String::new()).is_err());
        assert!(!paging.advance(Some(cut), String::new()).unwrap());
    }
}
