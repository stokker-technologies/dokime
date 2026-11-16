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
import subprocess
import sys
import uuid
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
import dokime_tools.test.policy

from .. import temporary
from ..build import paths
from ..process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #


class Root:
    """Own One Native Run's Scratch and Credential Namespace."""

    def __init__(self, environment: dict[str, str] | None, reports: Path) -> None:
        self._directory = temporary.Directory()
        self.path = self._directory.path
        self.reports = reports
        try:
            self.environment = dict(os.environ if environment is None else environment)
            self.environment.update({name: str(self.path) for name in ("TEMP", "TMP", "TMPDIR")})
            # Every Native Child of This Run Keeps Its Credentials in the Run's Own Namespace, Which Cleanup Removes.
            self.environment["DOKIME_CREDENTIAL_NAMESPACE"] = f"dokime-run-{uuid.uuid4().hex}"
        except BaseException:
            self._directory.close()
            raise

    def close(self) -> None:
        try:
            if sys.platform == "win32":
                for identity in self.path.rglob("backend-id"):
                    directory = dokime_tools.test.policy.root()
                    profile = self.environment.get("TEST_PROFILE", "bench")
                    output = paths.output(directory, "build", profile)
                    candidates = [output / "fixtures/host/dokime-host.exe", output / "bin/dokime-host.exe"]
                    host = next((candidate for candidate in candidates if candidate.is_file()), None)
                    try:
                        if host is None:
                            raise RuntimeError("Run Desktop Bench Tests: Missing Prepared Host")
                        subprocess.run(
                            [
                                str(host),
                                "--storage",
                                str(identity.parent),
                                "--credential-namespace",
                                self.environment["DOKIME_CREDENTIAL_NAMESPACE"],
                                "--remove-credentials",
                            ],
                            env=self.environment,
                            check=True,
                            creationflags=spawn_flags(),
                        )
                    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
                        self.reports.mkdir(parents=True, exist_ok=True)
                        retained = self.reports / self.path.parent.name
                        shutil.copytree(self.path, retained)
                        raise RuntimeError(f"Remove Scratch Credentials: {error}; Retained Root: {retained}") from error
            if self.environment.get("DOKIME_KEEP_EVIDENCE") == "1":
                self.reports.mkdir(parents=True, exist_ok=True)
                shutil.copytree(self.path, self.reports / self.path.parent.name)
        finally:
            self._directory.close()


# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
