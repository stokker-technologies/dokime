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
#[cfg(feature = "native")]
use crate::telemetry::TelemetryRow;
use crate::telemetry::{TelemetryContract, TelemetryStream};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
#[cfg(feature = "native")]
pub type Publish = for<'a> fn(
    &'a harmos::plugin::sidecar::Supervisor,
    &'a str,
    &'a std::collections::BTreeMap<String, crate::core::Value>,
) -> crate::transport::TransportFuture<'a, Result<(), String>>;

#[cfg(feature = "native")]
pub type Bind = fn(
    &mut harmos::plugin::sidecar::operations::WireTable<harmos::plugin::sidecar::Supervisor>,
    &str,
    &[RecordContract],
) -> Result<(), harmos::plugin::sidecar::operations::ServingError>;

/* === Structs === */
#[cfg(feature = "native")]
pub struct Streams;

pub struct StreamContracts;

pub struct Records;

pub struct Prompts;

#[cfg(feature = "native")]
pub struct Subscriptions;

#[cfg(feature = "native")]
pub struct Templates;

#[cfg(feature = "native")]
pub struct TemplateIdentifiers;

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

/* === Implementations === */
#[cfg(feature = "native")]
impl Visitor for Streams {
    type Entry = harmos::plugin::sidecar::inventory::publications::Entry;
}

#[cfg(feature = "native")]
impl<T: harmos::StreamSpec<Row: harmos::schema::Described>> Member<T> for Streams {
    const ENTRY: Self::Entry = harmos::plugin::sidecar::inventory::publications::Entry::of::<T>();
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
impl<P> Visitor for TemplateBindings<P> {
    type Entry = Bind;
}

#[cfg(feature = "native")]
impl<P, T> Member<T> for TemplateBindings<P>
where
    P: harmos::plugin::contract::Declared,
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
    P: harmos::plugin::contract::Declared,
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
    P: harmos::plugin::contract::Declared,
    T: TelemetryStream<Row: harmos::Message>,
{
    const ENTRY: Self::Entry = PublicationEntry {
        id: T::ID,
        publish: publish::<T, P>,
    };
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
    assert!(
        written == N,
        "a flattened catalog has exactly its leaf count"
    );
    flat
}

pub const fn joined<const N: usize>(id: &str, suffix: &str) -> [u8; N] {
    let (id, suffix) = (id.as_bytes(), suffix.as_bytes());
    assert!(
        id.len() + suffix.len() == N,
        "a joined id has exactly its length"
    );
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
    required: &[Node<harmos::operations::WireDeclaration>],
    granted: &[Node<harmos::plugin::contract::Requirement>],
    provided: &[Node<harmos::operations::WireDeclaration>],
) -> bool {
    let mut index = 0;
    while index < required.len() {
        let covered = match &required[index] {
            Node::Leaf(declaration) => {
                granted_id(granted, declaration.operation.id)
                    || provided_privately(provided, declaration.operation.id)
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

/* === Private Functions === */
const fn granted_id(granted: &[Node<harmos::plugin::contract::Requirement>], id: &str) -> bool {
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

const fn provided_privately(
    provided: &[Node<harmos::operations::WireDeclaration>],
    id: &str,
) -> bool {
    let mut index = 0;
    while index < provided.len() {
        let found = match &provided[index] {
            Node::Leaf(declaration) => {
                same(declaration.operation.id, id)
                    && matches!(
                        declaration.operation.visibility,
                        harmos::operations::Visibility::Private
                    )
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
    host: &'a harmos::plugin::sidecar::Supervisor,
    token: &'a str,
    values: &'a std::collections::BTreeMap<String, crate::core::Value>,
) -> crate::transport::TransportFuture<'a, Result<(), String>>
where
    T: TelemetryStream<Row: harmos::Message>,
    P: harmos::plugin::contract::Declared,
{
    Box::pin(async move {
        let row = <T::Row as TelemetryRow>::from_row(values).map_err(|error| error.to_string())?;
        host.publisher::<P>()
            .publish_on_context::<T>(&row, Some(token))
            .await
            .map_err(|error| error.to_string())
    })
}

/* === Tests === */
