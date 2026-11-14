# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Include the SDK License and Notice in Wheels."""

# === Imports === #

from pathlib import Path
from typing import Any

from hatchling.builders.hooks.plugin.interface import BuildHookInterface

# === Classes === #


class CustomBuildHook(BuildHookInterface):
    def initialize(self, version: str, build_data: dict[str, Any]) -> None:
        root = Path(self.root)
        for name in ("LICENSE", "NOTICE"):
            source = root / name if (root / name).is_file() else root.parents[1] / name
            build_data["force_include"][str(source)] = f"dokime/{name}"


# === Entry Point === #
