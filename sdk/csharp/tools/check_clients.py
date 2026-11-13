# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Check that the authoritative protocol provides client service definitions."""
# === Imports === #

# Standard Library Imports
import re
import sys
from pathlib import Path

# Third Party Imports
# Local Imports
# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #

# === Private Functions === #

# === Entry Point === #

root = Path(__file__).resolve().parents[1]
services = []
for protocol in (root / "../../proto/dokime/v1").resolve().rglob("*.proto"):
    services.extend(re.findall(r"^service\s+(\w+)\s*\{", protocol.read_text(), re.MULTILINE))
if not services:
    sys.exit(
        "Client coverage is blocked: proto/dokime/v1 has no service declarations. The protocol owner must supply every dokime.v1 service before client stubs can be generated and tested."
    )
generated = "\n".join(path.read_text(encoding="utf-8-sig") for path in (root / "obj/Release/net10.0").rglob("*Grpc.cs"))
missing = [service for service in services if f"class {service}Client" not in generated]
if missing:
    sys.exit("Missing generated clients: " + ", ".join(missing))
print(f"Generated client coverage: {len(services)} services")
