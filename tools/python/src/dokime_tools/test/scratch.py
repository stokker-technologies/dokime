# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Own Native Test Scratch Until Explicit Cleanup."""

# === Imports === #

# Standard Library Imports
import os
import shutil
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports

from .. import temporary

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class Root:
    """Own One Native Run's Scratch."""

    def __init__(self, environment: dict[str, str] | None, reports: Path) -> None:
        self._directory = temporary.Directory()
        self.path = self._directory.path
        self.reports = reports
        try:
            self.environment = dict(os.environ if environment is None else environment)
            self.environment.update({name: str(self.path) for name in ("TEMP", "TMP", "TMPDIR")})
        except BaseException:
            self._directory.close()
            raise

    def close(self) -> None:
        try:
            if self.environment.get("DOKIME_KEEP_EVIDENCE") == "1":
                self.reports.mkdir(parents=True, exist_ok=True)
                shutil.copytree(self.path, self.reports / self.path.parent.name)
        finally:
            self._directory.close()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
