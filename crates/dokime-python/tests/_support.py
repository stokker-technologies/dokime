# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# === Imports === #

# Standard Library Imports
from __future__ import annotations
from typing import ClassVar

# Third Party Imports

# Local Imports
from dokime import Suite
from dokime import Metadata
from dokime.core import CaseTemplate
from dokime.core.authoring import TestCaseTemplate
from dokime.core.authoring import parameter

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class AuthoringToken:
    def __str__(self) -> str:
        return "token-a"


class DynamicAuthoringTemplate(TestCaseTemplate):
    template_id: ClassVar[str | None] = "fixture.dynamic"

    token = parameter(AuthoringToken(), choices=[AuthoringToken()])


# === Public Functions === #


def fixture_suite() -> Suite:
    return Suite(
        metadata=Metadata("fixture.suite", "Fixture Suite"),
        templates=[
            CaseTemplate(
                metadata=Metadata("fixture.template", "Fixture Template"),
            )
        ],
    )


# === Private Functions === #

# === Entry Point === #
