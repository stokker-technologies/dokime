# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Remote Plugin Authoring, Generated Clients And Composed Documents."""

# === Imports === #

# Standard Library Imports
from importlib.metadata import version

# Third Party Imports
from harmos import Failure, Operation, operation

# Local Imports
from . import plot
from .authoring import Case, Criterion, Record, Step, Stream, Suite, Template, criterion, materialize, observer, step, stream, suite
from .client import Client, Cycle, MissingServiceError, connect
from .context import Context
from .documents import Document, Section, Summary
from .plugin import Endpoint, Plugin, serve

# Type Checking Imports

# === Public Exports === #

__all__ = [
    "Case",
    "Client",
    "Context",
    "Criterion",
    "Cycle",
    "Document",
    "Endpoint",
    "Failure",
    "MissingServiceError",
    "Operation",
    "Plugin",
    "Record",
    "Section",
    "Step",
    "Stream",
    "Suite",
    "Summary",
    "Template",
    "connect",
    "criterion",
    "materialize",
    "observer",
    "operation",
    "plot",
    "serve",
    "step",
    "stream",
    "suite",
]

# === Constants === #

__version__ = version("dokime")

# === Types === #

# === Classes === #

# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
