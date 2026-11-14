# ***
#
# dokime-rust-workspace
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Package Built Python SDK Distributions"""

# === Imports === #

import shutil
import tomllib
from pathlib import Path

from dokime_tools.build import wheel
from dokime_tools.package.staging import clear_output

# === Public Functions === #


def main() -> None:
    root = Path(__file__).resolve().parents[3]
    source = root / "target/build/dev/any/python"
    version = tomllib.loads((root / "sdk/python/pyproject.toml").read_text())["project"]["version"]
    artifacts = wheel.verify(root, source, version)
    output = root / "target/package/dev/any/python"
    clear_output(output)
    output.mkdir(parents=True, exist_ok=True)
    for artifact in artifacts:
        shutil.copy2(artifact, output / artifact.name)


# === Entry Point === #

if __name__ == "__main__":
    main()
