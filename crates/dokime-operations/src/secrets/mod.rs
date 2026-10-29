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

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::{Endpoint, MUTATION, ORDINARY};
use dokime_domain::model::secrets::catalog::{Metadata, Snapshot, Value};
use dokime_domain::model::secrets::transfer::{Archive, Choice, Preview as ImportPreview};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::operation(resources = [crate::hosting::Host<Lookup>], response = Value, id = "dokime.secrets.lookup", policy = ORDINARY,
    visibility = harmos::operations::Visibility::Public,
    grpc(package = "dokime.v1", service = "Secrets", method = "Lookup"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lookup {
    pub identifier: String,
}

#[harmos::operation(resources = [crate::hosting::Host<List>], response = Snapshot, id = "dokime.secrets.list", policy = ORDINARY,
    visibility = harmos::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Secrets", method = "List"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct List {}

#[harmos::message(skip_debug)]
#[harmos::operation(resources = [crate::hosting::Host<Put>], codec = manual, response = Snapshot, id = "dokime.secrets.put", policy = MUTATION,
    visibility = harmos::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Secrets", method = "Put"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Put {
    pub metadata: Metadata,
    pub value: Option<String>,
    pub generation: u64,
}

#[harmos::operation(resources = [crate::hosting::Host<Remove>], response = Snapshot, id = "dokime.secrets.remove", policy = MUTATION,
    visibility = harmos::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Secrets", method = "Remove"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Remove {
    pub identifiers: Vec<String>,
    pub generation: u64,
}

#[harmos::message(skip_debug)]
#[harmos::operation(resources = [crate::hosting::Host<Export>], codec = manual, response = Archive, id = "dokime.secrets.export", policy = MUTATION,
    visibility = harmos::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Secrets", method = "Export"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Export {
    pub identifiers: Vec<String>,
    pub generation: u64,
}

#[harmos::message(skip_debug)]
#[harmos::operation(resources = [crate::hosting::Host<Preview>], codec = manual, response = ImportPreview, id = "dokime.secrets.preview", policy = MUTATION,
    visibility = harmos::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Secrets", method = "Preview"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preview {
    pub data: Vec<u8>,
}

#[harmos::message(skip_debug)]
#[harmos::operation(resources = [crate::hosting::Host<Import>], codec = manual, response = Snapshot, id = "dokime.secrets.import", policy = MUTATION,
    visibility = harmos::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Secrets", method = "Import"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Import {
    pub data: Vec<u8>,
    pub choices: Vec<Choice>,
    pub generation: u64,
}

/* === Enums === */

#[harmos::catalog]
pub enum Operations {
    Lookup(Lookup),
    List(List),
    Put(Put),
    Remove(Remove),
    Export(Export),
    Preview(Preview),
    Import(Import),
}

/* === Traits === */

/* === Implementations === */

impl Endpoint for Lookup {
    const PATH: &'static str = "/dokime.v1.Secrets/Lookup";
}
impl Endpoint for List {
    const PATH: &'static str = "/dokime.v1.Secrets/List";
}
impl Endpoint for Put {
    const PATH: &'static str = "/dokime.v1.Secrets/Put";
}
impl Endpoint for Remove {
    const PATH: &'static str = "/dokime.v1.Secrets/Remove";
}
impl Endpoint for Export {
    const PATH: &'static str = "/dokime.v1.Secrets/Export";
}
impl Endpoint for Preview {
    const PATH: &'static str = "/dokime.v1.Secrets/Preview";
}
impl Endpoint for Import {
    const PATH: &'static str = "/dokime.v1.Secrets/Import";
}

impl std::fmt::Debug for Put {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("PutSecret(<redacted>)")
    }
}

impl std::fmt::Debug for Export {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ExportSecretFile(<redacted>)")
    }
}
impl std::fmt::Debug for Preview {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("PreviewSecretFile(<redacted>)")
    }
}
impl std::fmt::Debug for Import {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ImportSecretFile(<redacted>)")
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

/* === --- === Runtime Module === --- === */

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(Lookup, List, Put, Remove, Export, Preview, Import);
}
