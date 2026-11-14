# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Check Frozen Service Stubs Against the Authoritative Protocol."""
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

root: Path = Path(__file__).resolve().parents[1]
services: list[tuple[str, str]] = []
for protocol in (root / "../../proto/dokime").resolve().rglob("*.proto"):
    source = protocol.read_text()
    package_match: re.Match[str] | None = re.search(r"^package\s+([\w.]+);", source, re.MULTILINE)
    if package_match is None:
        sys.exit(f"Check C# Clients: Missing Package in {protocol}")
    package: str = package_match.group(1)
    services.extend((package, service) for service in re.findall(r"^service\s+(\w+)\s*\{", source, re.MULTILINE))
if not services:
    sys.exit("Client coverage is blocked: proto/dokime has no service declarations.")

generated: list[str] = [path.read_text(encoding="utf-8-sig") for path in (root / "src/Dokime/Generated/Wire").rglob("*Grpc.cs")]
missing: list[str] = [
    f"{package}.{service}"
    for package, service in services
    if not any(
        f'__ServiceName = "{package}.{service}";' in source
        and f"internal static partial class {service}" in source
        and f"class {service}Client" in source
        for source in generated
    )
]
if missing:
    sys.exit("Missing generated clients: " + ", ".join(missing))
print(f"Generated client coverage: {len(services)} services")
