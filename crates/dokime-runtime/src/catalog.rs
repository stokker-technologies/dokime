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

pub use harmos::catalog::{Catalog, Contains, InertCatalog, Member, Node, Visitor};

/* === Imports === */

// Standard Library Imports
#[cfg(feature = "native")]
use std::marker::PhantomData;

// Third Party Imports

// Local Imports
#[cfg(feature = "native")]
use crate::definition::DeclaredTemplate;

#[cfg(feature = "native")]
use crate::execution::SidecarTemplate;
use crate::records::{DeclaredRecord, RecordContract};
use crate::telemetry::{TelemetryContract, TelemetryStream};

/* === Constants === */

#[cfg(feature = "native")]
const UNSET: Publication = Publication {
    id: "",
    metadata_json: "",
    row_metadata_json: "[]",
    schema: 0,
    window: 0,
    row: harmos::schema::declaration::Definition::of::<()>(),
};

/* === Macros === */

/* === Type Aliases === */

#[cfg(feature = "native")]
pub type Publication = harmos::sidecar::inventory::publications::Entry;

#[cfg(feature = "native")]
type Observer = harmos::sidecar::inventory::observers::Entry;

#[cfg(feature = "native")]
pub type Publish = for<'a> fn(
    &'a harmos::sidecar::Supervisor,
    &'a str,
    &'a [crate::execution::StampedRow],
) -> crate::transport::TransportFuture<'a, Result<(), String>>;

#[cfg(feature = "native")]
pub type Bind =
    fn(&mut harmos::sidecar::operations::WireTable, &str, &[RecordContract]) -> Result<(), harmos::sidecar::operations::ServingError>;

/* === Structs === */

#[cfg(feature = "native")]
pub struct Streams;

pub struct StreamContracts;

pub struct Records;

pub struct Prompts;

#[cfg(feature = "native")]
pub struct Subscriptions;

#[cfg(feature = "native")]
pub struct TemplateStreams;

#[cfg(feature = "native")]
pub struct SuiteStreams;

#[cfg(feature = "native")]
pub struct Templates;

#[cfg(feature = "native")]
pub struct TemplateIdentifiers;

#[cfg(feature = "native")]
pub struct TemplateIdentifierNodes;

#[cfg(feature = "native")]
pub struct SuiteTemplateIdentifiers;

pub struct SuiteIdentifiers;

pub struct RecordIdentifiers;

pub struct StreamIdentifiers;

pub struct Suites;

#[cfg(feature = "native")]
pub struct TemplateBindings<P>(PhantomData<fn() -> P>);

#[cfg(feature = "native")]
pub struct TemplateOperations;

#[cfg(feature = "native")]
pub struct SuiteBindings<P>(PhantomData<fn() -> P>);

#[cfg(feature = "native")]
pub struct SuiteOperations;

#[cfg(feature = "native")]
pub struct Publications<P>(PhantomData<fn() -> P>);

#[cfg(feature = "native")]
pub struct TemplateRunners;

#[cfg(feature = "native")]
pub struct SuiteRunners;

pub struct Listeners;

#[cfg(feature = "native")]
pub struct TemplateEntry {
    pub declaration: fn() -> crate::definition::TestCaseTemplate,
    pub records: fn() -> Vec<RecordContract>,
    pub streams: fn() -> Vec<TelemetryContract>,
}

#[cfg(feature = "native")]
pub struct PublicationEntry {
    pub id: &'static str,
    pub publish: Publish,
}

#[derive(Clone, Copy)]
pub struct ListenerEntry {
    pub operation: &'static str,
    pub record: &'static str,
}

/* === Enums === */

/* === Traits === */

pub trait IdentifierCatalog<V: Visitor<Entry = Node<&'static str>>> {
    const IDENTIFIERS: &'static [Node<&'static str>];
}

/* === Implementations === */

impl<V: Visitor<Entry = Node<&'static str>>> IdentifierCatalog<V> for () {
    const IDENTIFIERS: &'static [Node<&'static str>] = &[];
}

#[cfg(feature = "native")]
impl Visitor for Streams {
    type Entry = Node<Publication>;
}

// A stream the plugin only reads is no part of what it publishes.
#[cfg(feature = "native")]
impl<T: TelemetryStream<Row: harmos::schema::Described>> Member<T> for Streams {
    const ENTRY: Self::Entry = match T::PUBLISHED {
        true => Node::Leaf(Publication::of::<T>()),
        false => Node::Nested(&[]),
    };
}

#[cfg(feature = "native")]
impl Visitor for TemplateStreams {
    type Entry = Node<Publication>;
}

#[cfg(feature = "native")]
impl<T: SidecarTemplate> Member<T> for TemplateStreams {
    const ENTRY: Self::Entry = Node::Nested(T::STREAMS);
}

#[cfg(feature = "native")]
impl Visitor for SuiteStreams {
    type Entry = Node<Publication>;
}

#[cfg(feature = "native")]
impl<T: crate::definition::DeclaredSuite> Member<T> for SuiteStreams {
    const ENTRY: Self::Entry = Node::Nested(T::STREAMS);
}

impl Visitor for StreamContracts {
    type Entry = fn() -> TelemetryContract;
}

impl<T: TelemetryStream> Member<T> for StreamContracts {
    const ENTRY: Self::Entry = T::contract;
}

impl Visitor for Records {
    type Entry = fn() -> RecordContract;
}

impl<T: DeclaredRecord> Member<T> for Records {
    const ENTRY: Self::Entry = T::contract;
}

impl Visitor for Prompts {
    type Entry = fn() -> crate::prompts::PromptDefinition;
}

impl<T: crate::prompts::DeclaredPrompt> Member<T> for Prompts {
    const ENTRY: Self::Entry = T::definition;
}

#[cfg(feature = "native")]
impl Visitor for Subscriptions {
    type Entry = fn() -> crate::definition::Subscription;
}

#[cfg(feature = "native")]
impl<T: crate::execution::Listenable> Member<T> for Subscriptions {
    const ENTRY: Self::Entry = T::subscription;
}

impl Visitor for Suites {
    type Entry = fn() -> crate::domains::TestSuiteDefinition;
}

impl<T: crate::definition::DeclaredSuite> Member<T> for Suites {
    const ENTRY: Self::Entry = T::definition;
}

#[cfg(feature = "native")]
impl Visitor for Templates {
    type Entry = TemplateEntry;
}

#[cfg(feature = "native")]
impl<T: DeclaredTemplate + SidecarTemplate> Member<T> for Templates {
    const ENTRY: Self::Entry = TemplateEntry {
        declaration: <T as DeclaredTemplate>::declaration,
        records: <T as DeclaredTemplate>::record_contracts,
        streams: <T as DeclaredTemplate>::stream_contracts,
    };
}

#[cfg(feature = "native")]
impl Visitor for TemplateIdentifiers {
    type Entry = &'static str;
}

#[cfg(feature = "native")]
impl<T: DeclaredTemplate> Member<T> for TemplateIdentifiers {
    const ENTRY: Self::Entry = T::IDENTIFIER;
}

#[cfg(feature = "native")]
impl Visitor for TemplateIdentifierNodes {
    type Entry = Node<&'static str>;
}

#[cfg(feature = "native")]
impl<T: DeclaredTemplate> Member<T> for TemplateIdentifierNodes {
    const ENTRY: Self::Entry = Node::Leaf(T::IDENTIFIER);
}

#[cfg(feature = "native")]
impl Visitor for SuiteTemplateIdentifiers {
    type Entry = Node<&'static str>;
}

#[cfg(feature = "native")]
impl<T: crate::definition::DeclaredSuite> Member<T> for SuiteTemplateIdentifiers {
    const ENTRY: Self::Entry = Node::Nested(T::TEMPLATE_IDENTIFIERS);
}

impl Visitor for SuiteIdentifiers {
    type Entry = Node<&'static str>;
}

impl<T: crate::definition::DeclaredSuite> Member<T> for SuiteIdentifiers {
    const ENTRY: Self::Entry = Node::Leaf(T::ID);
}

impl Visitor for RecordIdentifiers {
    type Entry = Node<&'static str>;
}

impl<T: DeclaredRecord> Member<T> for RecordIdentifiers {
    const ENTRY: Self::Entry = Node::Leaf(T::RECORD_TYPE);
}

impl Visitor for StreamIdentifiers {
    type Entry = Node<&'static str>;
}

impl<T: TelemetryStream> Member<T> for StreamIdentifiers {
    const ENTRY: Self::Entry = Node::Leaf(T::ID);
}

#[cfg(feature = "native")]
impl<P> Visitor for TemplateBindings<P> {
    type Entry = Bind;
}

#[cfg(feature = "native")]
impl<P, T> Member<T> for TemplateBindings<P>
where
    P: harmos::plugin::Declared,
    T: SidecarTemplate,
{
    const ENTRY: Self::Entry = {
        assert!(
            required_by(T::REQUIRED, P::REQUIRES, P::PROVIDES),
            "a template requires an operation its plugin neither names in `requires(...)` nor provides privately"
        );
        crate::execution::bind::<T, P>
    };
}

#[cfg(feature = "native")]
impl Visitor for TemplateRunners {
    type Entry = fn(&str, &[RecordContract]) -> std::sync::Arc<dyn crate::execution::Runnable>;
}

#[cfg(feature = "native")]
impl<T: SidecarTemplate> Member<T> for TemplateRunners {
    const ENTRY: Self::Entry = crate::execution::runner::<T>;
}

#[cfg(feature = "native")]
impl Visitor for SuiteRunners {
    type Entry = fn() -> Vec<std::sync::Arc<dyn crate::execution::Runnable>>;
}

#[cfg(feature = "native")]
impl<T: crate::definition::DeclaredSuite> Member<T> for SuiteRunners {
    const ENTRY: Self::Entry = T::runners;
}

#[cfg(feature = "native")]
impl Visitor for TemplateOperations {
    type Entry = Node<harmos::operations::WireDeclaration>;
}

#[cfg(feature = "native")]
impl<T: SidecarTemplate> Member<T> for TemplateOperations {
    const ENTRY: Self::Entry = Node::Nested(T::OPERATIONS);
}

#[cfg(feature = "native")]
impl Visitor for SuiteOperations {
    type Entry = Node<harmos::operations::WireDeclaration>;
}

#[cfg(feature = "native")]
impl<T: crate::definition::DeclaredSuite> Member<T> for SuiteOperations {
    const ENTRY: Self::Entry = Node::Nested(T::OPERATIONS);
}

#[cfg(feature = "native")]
impl<P> Visitor for SuiteBindings<P> {
    type Entry = Bind;
}

#[cfg(feature = "native")]
impl<P, T> Member<T> for SuiteBindings<P>
where
    P: harmos::plugin::Declared,
    T: crate::definition::DeclaredSuite,
{
    const ENTRY: Self::Entry = T::bind::<P>;
}

#[cfg(feature = "native")]
impl<P> Visitor for Publications<P> {
    type Entry = PublicationEntry;
}

#[cfg(feature = "native")]
impl<P, T> Member<T> for Publications<P>
where
    P: harmos::plugin::Declared,
    T: TelemetryStream<Row: harmos::Message>,
{
    const ENTRY: Self::Entry = PublicationEntry { id: T::ID, publish: publish::<T, P> };
}

impl Visitor for Listeners {
    type Entry = ListenerEntry;
}

#[cfg(feature = "native")]
impl<T: crate::plugin::Listener> Member<T> for Listeners {
    const ENTRY: Self::Entry = ListenerEntry {
        operation: T::ID,
        record: <T::Record as DeclaredRecord>::RECORD_TYPE,
    };
}

/* === Public Functions === */

pub fn entries<C, V>() -> Vec<V::Entry>
where
    C: Catalog<V>,
    V: Visitor,
    V::Entry: Clone,
{
    use harmos::catalog::Projection;
    let mut collected = Vec::new();
    let _ = <C as Catalog<V>>::ENTRIES.try_for_each::<()>(|entry| {
        collected.push(entry.clone());
        Ok(())
    });
    collected
}

pub const fn flatten<E: Copy, const N: usize>(nodes: &[Node<E>], fill: E) -> [E; N] {
    let mut flat = [fill; N];
    let written = write(nodes, &mut flat, 0);
    assert!(written == N, "a flattened catalog has exactly its leaf count");
    flat
}

pub const fn joined<const N: usize>(id: &str, suffix: &str) -> [u8; N] {
    let (id, suffix) = (id.as_bytes(), suffix.as_bytes());
    assert!(id.len() + suffix.len() == N, "a joined id has exactly its length");
    let mut bytes = [0; N];
    let mut index = 0;
    while index < id.len() {
        bytes[index] = id[index];
        index += 1;
    }
    let mut offset = 0;
    while offset < suffix.len() {
        bytes[index + offset] = suffix[offset];
        offset += 1;
    }
    bytes
}

pub const fn required_by(
    required: &[Node<harmos::operations::WireDeclaration>], granted: &[Node<harmos::catalog::RequirementEntry>],
    provided: &[Node<harmos::operations::WireDeclaration>],
) -> bool {
    let mut index = 0;
    while index < required.len() {
        let covered = match &required[index] {
            Node::Leaf(declaration) => {
                granted_id(granted, declaration.operation.id) || provided_privately(provided, declaration.operation.id)
            }
            Node::Nested(nested) => required_by(nested, granted, provided),
        };
        if !covered {
            return false;
        }
        index += 1;
    }
    true
}

#[cfg(feature = "native")]
pub const fn publication_count(streams: &[Node<Publication>], observers: &[Node<Observer>]) -> usize {
    harmos::catalog::count(streams) + observed_count(observers)
}

#[cfg(feature = "native")]
pub const fn publications<const N: usize>(streams: &[Node<Publication>], observers: &[Node<Observer>]) -> [Publication; N] {
    let mut gathered = [UNSET; N];
    let at = write(streams, &mut gathered, 0);
    let written = write_observed(observers, &mut gathered, at);
    assert!(written == N, "a plugin's publications are exactly its declared streams");
    gathered
}

#[cfg(feature = "native")]
pub const fn distinct_publications(declared: &[Publication]) -> usize {
    let mut distinct = 0;
    let mut index = 0;
    while index < declared.len() {
        match earlier(declared, index) {
            Some(first) => assert!(
                same_publication(&declared[first], &declared[index]),
                "two declarations of the plugin name one stream id with different rows, schema versions, windows or metadata"
            ),
            None => distinct += 1,
        }
        index += 1;
    }
    distinct
}

pub const fn assert_unique(identifiers: &[&str]) {
    let mut index = 0;
    while index < identifiers.len() {
        let mut earlier = 0;
        while earlier < index {
            if same(identifiers[earlier], identifiers[index]) {
                panic!("{}", identifiers[index]);
            }
            earlier += 1;
        }
        index += 1;
    }
}

#[cfg(feature = "native")]
pub const fn deduplicated<const N: usize>(declared: &[Publication]) -> [Publication; N] {
    let mut distinct = [UNSET; N];
    let mut written = 0;
    let mut index = 0;
    while index < declared.len() {
        if earlier(declared, index).is_none() {
            distinct[written] = declared[index];
            written += 1;
        }
        index += 1;
    }
    assert!(written == N, "a deduplicated grant has exactly its distinct streams");
    distinct
}

/* === Private Functions === */

const fn granted_id(granted: &[Node<harmos::catalog::RequirementEntry>], id: &str) -> bool {
    let mut index = 0;
    while index < granted.len() {
        let found = match &granted[index] {
            Node::Leaf(requirement) => same(requirement.declaration.operation.id, id),
            Node::Nested(nested) => granted_id(nested, id),
        };
        if found {
            return true;
        }
        index += 1;
    }
    false
}

const fn provided_privately(provided: &[Node<harmos::operations::WireDeclaration>], id: &str) -> bool {
    let mut index = 0;
    while index < provided.len() {
        let found = match &provided[index] {
            Node::Leaf(declaration) => {
                same(declaration.operation.id, id) && matches!(declaration.operation.visibility, harmos::operations::Visibility::Private)
            }
            Node::Nested(nested) => provided_privately(nested, id),
        };
        if found {
            return true;
        }
        index += 1;
    }
    false
}

#[cfg(feature = "native")]
const fn observed_count(observers: &[Node<Observer>]) -> usize {
    let mut count = 0;
    let mut index = 0;
    while index < observers.len() {
        count += match &observers[index] {
            Node::Leaf(observer) => observer.streams.len(),
            Node::Nested(nested) => observed_count(nested),
        };
        index += 1;
    }
    count
}

#[cfg(feature = "native")]
const fn write_observed<const N: usize>(observers: &[Node<Observer>], flat: &mut [Publication; N], at: usize) -> usize {
    let mut cursor = at;
    let mut index = 0;
    while index < observers.len() {
        match &observers[index] {
            Node::Leaf(observer) => {
                let mut stream = 0;
                while stream < observer.streams.len() {
                    flat[cursor] = observer.streams[stream];
                    cursor += 1;
                    stream += 1;
                }
            }
            Node::Nested(nested) => cursor = write_observed(nested, flat, cursor),
        }
        index += 1;
    }
    cursor
}

#[cfg(feature = "native")]
const fn earlier(declared: &[Publication], index: usize) -> Option<usize> {
    let mut candidate = 0;
    while candidate < index {
        if same(declared[candidate].id, declared[index].id) {
            return Some(candidate);
        }
        candidate += 1;
    }
    None
}

// The row's fields are reached only through function pointers, which a
// constant cannot call; its type name stands for them here, and admission
// compares the fields themselves.
#[cfg(feature = "native")]
const fn same_publication(left: &Publication, right: &Publication) -> bool {
    left.schema == right.schema
        && left.window == right.window
        && same(left.row.name, right.row.name)
        && same(left.metadata_json, right.metadata_json)
        && same(left.row_metadata_json, right.row_metadata_json)
}

const fn same(left: &str, right: &str) -> bool {
    let (left, right) = (left.as_bytes(), right.as_bytes());
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}

const fn write<E: Copy, const N: usize>(nodes: &[Node<E>], flat: &mut [E; N], at: usize) -> usize {
    let mut cursor = at;
    let mut index = 0;
    while index < nodes.len() {
        match &nodes[index] {
            Node::Leaf(entry) => {
                flat[cursor] = *entry;
                cursor += 1;
            }
            Node::Nested(nested) => cursor = write(nested, flat, cursor),
        }
        index += 1;
    }
    cursor
}

#[cfg(feature = "native")]
fn publish<'a, T, P>(
    host: &'a harmos::sidecar::Supervisor, token: &'a str, rows: &'a [crate::execution::StampedRow],
) -> crate::transport::TransportFuture<'a, Result<(), String>>
where
    T: TelemetryStream<Row: harmos::Message>,
    P: harmos::plugin::Declared,
{
    Box::pin(async move {
        let publisher = host.publisher::<P>();
        let mut batch = publisher.batch::<T>();
        // Each Row Travels As The Bytes Its Producer Encoded Once; The Host
        // Validates Them Against The Stream Schema At Its Ingest Boundary.
        for row in rows {
            let bytes = row.storage_payload.as_ref().ok_or_else(|| "typed stream publication requires original row bytes".to_owned())?;
            batch.push_encoded_at(bytes.clone(), row.published_unix_micros);
        }
        publisher.publish_batch_on_context::<T>(&batch, Some(token)).await.map_err(|error| error.to_string())
    })
}

/* === Tests === */
