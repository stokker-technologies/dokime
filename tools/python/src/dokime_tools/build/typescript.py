# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Typescript Build Task"""

# === Imports === #

# Standard Library Imports
# Third Party Imports
# Local Imports
from dokime_tools.build import configuration, frontend
from dokime_tools.prepare import bindings, generated

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def build(profile: str) -> list[str]:
    """Generate The GUI Message Modules From Protobuf And Build The Live Frontend."""
    messages = configuration.messages(profile)
    bindings.generate(messages, check=True)
    generated.generate(messages, check=True)
    generated.stage(profile)
    built = frontend.build(profile)
    files = [path for path in built.rglob("*") if path.is_file()]
    return configuration.relative(files)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    raise SystemExit(configuration.invoke(build))
