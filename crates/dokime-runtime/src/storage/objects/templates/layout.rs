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
use super::documents;
use crate::storage::{Error, Result, objects::metadata};

/* === Constants === */
const MAX_DEPTH: usize = 64;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq)]
pub(super) struct Layout {
    nodes: Vec<Node>,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
struct Node {
    children: u32,
    section: Option<Section>,
    card: Option<documents::Placement>,
}
#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Section {
    #[serde(default)]
    node_id: String,
    #[serde(default)]
    metadata: Option<metadata::Row>,
    #[serde(default)]
    repeat: Option<documents::Repeat>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl TryFrom<&crate::document::DocumentLayout> for Layout {
    type Error = Error;

    fn try_from(value: &crate::document::DocumentLayout) -> Result<Self> {
        let mut nodes = Vec::new();
        for node in &value.nodes {
            append_node(node, &mut nodes, 0)?;
        }
        Ok(Self { nodes })
    }
}

impl TryFrom<Layout> for crate::document::DocumentLayout {
    type Error = Error;

    fn try_from(row: Layout) -> Result<Self> {
        let mut source = row.nodes.into_iter();
        let mut nodes = Vec::new();
        while source.len() > 0 {
            nodes.push(consume_node(&mut source, 0)?);
        }
        Ok(Self { nodes })
    }
}

impl Serialize for Layout {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let layout: crate::document::DocumentLayout = self.clone().try_into().map_err(serde::ser::Error::custom)?;
        layout.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for Layout {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let source = crate::document::DocumentLayout::deserialize(deserializer)?;
        Self::try_from(&source).map_err(serde::de::Error::custom)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn append_node(value: &crate::document::DocumentNode, nodes: &mut Vec<Node>, depth: usize) -> Result<()> {
    if depth > MAX_DEPTH {
        return Err(Error::Invalid("document layout exceeds depth limit"));
    }
    match value {
        crate::document::DocumentNode::Card(card) => nodes.push(Node {
            card: Some(documents::Placement::try_from(card)?),
            ..Default::default()
        }),
        crate::document::DocumentNode::Section(section) => {
            let definition = Section {
                node_id: section.node_id.as_str().to_owned(),
                metadata: Some(metadata::Row::try_from(&section.metadata)?),
                repeat: section.repeat.as_ref().map(documents::Repeat::try_from).transpose()?,
            };
            nodes.push(Node {
                children: u32::try_from(section.children.len()).map_err(|_| Error::Invalid("too many document children"))?,
                section: Some(definition),
                card: None,
            });
            for child in &section.children {
                append_node(child, nodes, depth + 1)?;
            }
        }
    }
    Ok(())
}

fn consume_node(nodes: &mut std::vec::IntoIter<Node>, depth: usize) -> Result<crate::document::DocumentNode> {
    use crate::document::{DocumentNode, DocumentSectionNode};
    if depth > MAX_DEPTH {
        return Err(Error::Invalid("document layout exceeds depth limit"));
    }
    let node = nodes.next().ok_or(Error::Invalid("document layout child is missing"))?;
    if node.children as usize > nodes.len() {
        return Err(Error::Invalid("document children exceed remaining nodes"));
    }
    match (node.section, node.card) {
        (Some(section), None) => {
            let mut children = Vec::new();
            children.try_reserve_exact(node.children as usize).map_err(|_| Error::Invalid("document children allocation failed"))?;
            for _ in 0..node.children {
                children.push(consume_node(nodes, depth + 1)?);
            }
            let section = DocumentSectionNode {
                node_id: dokime_domain::model::identity::Identifier::new(section.node_id)?,
                metadata: section.metadata.ok_or(Error::Invalid("document section metadata is missing"))?.try_into()?,
                repeat: section.repeat.map(TryInto::try_into).transpose()?,
                children,
            };
            Ok(DocumentNode::Section(Box::new(section)))
        }
        (None, Some(card)) if node.children == 0 => Ok(DocumentNode::Card(card.try_into()?)),
        _ => Err(Error::Invalid("document node fields or arity differ")),
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime_domain::model::document::definition::{DocumentCardPlacement, DocumentLayout, DocumentNode};
    use dokime_domain::model::identity::Identifier;

    #[test]
    fn admits_a_valid_layout_above_the_former_node_limit() {
        let card_id = Identifier::new("card").unwrap();
        let card = DocumentCardPlacement { card_id, layout: Default::default() };
        let card = DocumentNode::Card(card);
        let source = DocumentLayout { nodes: vec![card; 4097] };
        let row = Layout::try_from(&source).unwrap();

        assert_eq!(row.nodes.len(), 4097);
        let decoded = DocumentLayout::try_from(row).unwrap();
        assert_eq!(decoded, source);
    }
}
