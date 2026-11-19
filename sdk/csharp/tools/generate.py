# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Generate or Verify the SDK-Owned C# Projection from Frozen Descriptors."""

# === Imports === #

# Standard Library Imports
import argparse
import shutil
import tempfile
from pathlib import Path

# isort: split

# Third Party Imports

# isort: split

# Local Imports
from dokime_tools.build.protocol import configuration
from harmos_tools.build.resources import sdk
from projection import project

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def main() -> None:
    root = Path(__file__).resolve().parents[3]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    arguments = parser.parse_args()
    output = Path(tempfile.mkdtemp(prefix="dokime-csharp-"))
    try:
        sdk.generate(root=output, descriptors=root / "crates/dokime-protocol/src/descriptors.bin", configuration=configuration(root))
        project(output)
        relative = Path("sdk/csharp/src/Dokime/Generated")
        expected = {path.relative_to(output) for path in (output / relative).rglob("*.cs")}
        existing = {path.relative_to(root) for path in (root / relative).rglob("*.cs")}
        if arguments.write:
            for path in existing - expected:
                (root / path).unlink()
            for path in expected:
                (root / path).parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(output / path, root / path)
            return
        differences = sorted(str(path) for path in expected ^ existing)
        differences.extend(str(path) for path in sorted(expected & existing) if (root / path).read_bytes() != (output / path).read_bytes())
        if differences:
            raise ValueError("Check C# Generation: Run mise -C sdk/csharp run generate -- --write\n" + "\n".join(differences))
    finally:
        shutil.rmtree(output)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
