# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

"""Project Generated C# Carriers Behind the Authored SDK Surface."""

# === Imports === #

import re
from pathlib import Path

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def project(root: Path) -> None:
    generated = root / "sdk/csharp/src/Dokime/Generated"
    values = ""
    for path in generated.rglob("*.cs"):
        source = path.read_text(encoding="utf-8-sig")
        # Suite Is an Author Base; the Protocol Value Has One Internal Name.
        source = re.sub(r"\bSuite\b", "SuiteDefinition", source)
        source = source.replace("global::Dokime.Clients.ResultStatus", "global::Dokime.ResultStatus")
        status = re.search(r"  public enum ResultStatus \{[^}]+\}", source)
        if status is not None:
            values = status[0].replace("pbr::OriginalName", "OriginalName")
            source = source[: status.start()] + source[status.end() :]
        if path.name == "Client.g.cs":
            source = source.replace("namespace Dokime;", "namespace Dokime.Transport;")
            source = source.replace("public sealed class", "internal sealed class")
        if path.name == "Host.g.cs":
            source = source.replace("public sealed", "internal sealed")
            source = source.replace("public enum", "internal enum")
            source = source.replace("public static class", "internal static class")
        if re.search(r"namespace Dokime\.Clients(?:[.;])", source):
            source = re.sub(r"(?m)^  public (sealed partial class|static partial class|enum) ", r"  internal \1 ", source)
        path.write_text(source, encoding="utf-8", newline="\n")
    if not values:
        raise ValueError("Project C# Values: Restore the Protocol ResultStatus Enum")
    (generated / "Values.g.cs").write_text(
        "// @generated from protocol descriptors; do not edit.\nusing Google.Protobuf.Reflection;\nnamespace Dokime;\n" + values + "\n",
        encoding="utf-8",
        newline="\n",
    )


# === Private Functions === #

# === Entry Point === #
