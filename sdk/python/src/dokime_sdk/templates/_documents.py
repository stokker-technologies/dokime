# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Value Composition Of Canonical Document Messages."""

# === Imports === #

# Standard Library Imports
import re
from typing import Self

# Third Party Imports
# Local Imports
from .._generated.proto.model.v1.documents_pb2 import EVIDENCE_CARD_RECORDS, EVIDENCE_CARD_TRACEABILITY, Card, CardLayout
from .._generated.proto.model.v1.documents_pb2 import Document as DocumentMessage
from .._generated.proto.model.v1.documents_pb2 import Section as SectionMessage
from .._generated.proto.model.v1.documents_pb2 import Summary as SummaryMessage
from .._generated.proto.model.v1.expressions_pb2 import DataExpression
from .._generated.proto.model.v1.metadata_pb2 import Metadata
from ._plots import Expression, Plot, _expression

# Type Checking Imports

# === Public Exports === #


# === Constants === #

# === Types === #

# === Classes === #


class Document:
    def __init__(self, label: str) -> None:
        self.message = DocumentMessage(metadata=metadata(label))

    @property
    def label(self) -> str:
        return self.message.metadata.label

    def id(self, identifier: str) -> Self:
        self.message.metadata.identifier = identifier
        return self

    def section(self, section: "Section") -> Self:
        if not isinstance(section, Section):
            raise TypeError("Document.section requires a Section value")
        self.message.sections.add().CopyFrom(section.message)
        return self


class Section:
    def __init__(self, label: str) -> None:
        self.message = SectionMessage(metadata=metadata(label))

    def id(self, identifier: str) -> Self:
        self.message.metadata.identifier = identifier
        return self

    def plot(self, plot: Plot) -> Self:
        if not isinstance(plot, Plot):
            raise TypeError("Section.plot requires a Plot value")
        value = type(plot.message)()
        value.CopyFrom(plot.message)
        if not value.metadata.label:
            expression = value.z if value.HasField("z") else value.y[0] if value.y else value.x
            label = expression_label(expression)
            identifier = value.metadata.identifier
            value.metadata.CopyFrom(metadata(label))
            if identifier:
                value.metadata.identifier = identifier
        self.message.cards.add(metadata=value.metadata, plot=value)
        return self

    def section(self, section: "Section") -> Self:
        if not isinstance(section, Section):
            raise TypeError("Section.section requires a Section value")
        self.message.sections.add().CopyFrom(section.message)
        return self

    def summary(self, summary: "Summary", *, label: str = "Summary") -> Self:
        if not isinstance(summary, Summary):
            raise TypeError("Section.summary requires a Summary value")
        self.message.cards.add(metadata=metadata(label), summary=summary.message)
        return self

    def text(self, label: str, markdown: str) -> Self:
        self.message.cards.add(metadata=metadata(label), markdown=markdown)
        return self

    def card(self, card: Card) -> Self:
        if not isinstance(card, Card):
            raise TypeError("Section.card requires a Card value")
        self.message.cards.add().CopyFrom(card)
        return self

    def traceability(self, *, label: str = "Traceability") -> Self:
        self.message.cards.add(metadata=metadata(label), evidence=EVIDENCE_CARD_TRACEABILITY)
        return self

    def repeat(self, value: Expression, *, label_template: str = "", order_by: Expression | None = None) -> Self:
        self.message.repeat.value.CopyFrom(_expression(value))
        self.message.repeat.label_template = label_template
        self.message.repeat.ClearField("order_by")
        if order_by is not None:
            self.message.repeat.order_by.CopyFrom(_expression(order_by))
        return self


class Summary:
    def __init__(self) -> None:
        self.message = SummaryMessage()

    def tile(self, label: str, value: Expression, *, unit: str = "") -> Self:
        self.message.tiles.add(metadata=metadata(label), value=_expression(value), unit=unit)
        return self


# === Public Functions === #


def plot_card(plot: Plot, columns: int = 12) -> Card:
    return Card(metadata=plot.message.metadata, plot=plot.message, layout=CardLayout(columns=columns))


def records_card(identifier: str) -> Card:
    return Card(metadata=Metadata(identifier=identifier, label="Records"), evidence=EVIDENCE_CARD_RECORDS)


def metadata(label: str) -> Metadata:
    return Metadata(identifier=".".join(part.lower() for part in re.split(r"[\W_]+", label) if part), label=label)


# === Private Functions === #


def expression_label(value: DataExpression) -> str:
    if value.label:
        return value.label
    kind = value.WhichOneof("kind")
    if kind in ("stream", "record"):
        return getattr(value, kind).field
    if kind in ("aggregate", "transform"):
        return expression_label(getattr(value, kind).input)
    return "Value"


# === Entry Point === #
