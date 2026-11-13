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
for protocol in (root / "../../proto/dokime").resolve().rglob("*.proto"):
    source = protocol.read_text()
    package = re.search(r"^package\s+([\w.]+);", source, re.MULTILINE).group(1)
    namespace = re.search(r'^option csharp_namespace = "([\w.]+)";', source, re.MULTILINE)
    namespace = namespace.group(1) if namespace else ".".join(part.title() for part in package.split("."))
    services.extend((package, namespace, service) for service in re.findall(r"^service\s+(\w+)\s*\{", source, re.MULTILINE))
if not services:
    sys.exit("Client coverage is blocked: proto/dokime has no service declarations.")

generated = [path.read_text(encoding="utf-8-sig") for path in (root / "obj/Release/net10.0").rglob("*Grpc.cs")]
missing = [
    f"{package}.{service}"
    for package, namespace, service in services
    if not any(f"namespace {namespace} {{" in source and f"class {service}Client" in source for source in generated)
]
if missing:
    sys.exit("Missing generated clients: " + ", ".join(missing))
print(f"Generated client coverage: {len(services)} services")
