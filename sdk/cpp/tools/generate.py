# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Check C++ Descriptor Projections Without Changing Frozen Outputs."""

# === Imports === #

# Standard Library Imports
import argparse
import re
import shutil
import subprocess
import tempfile
from pathlib import Path

# isort: split

# Third Party Imports
from harmos_tools.build.resources import sdk

# isort: split

# Local Imports
from dokime_tools.build.protocol import configuration, host_resources
from dokime_tools.process import spawn_flags

# Type Checking Imports

# === Public Exports === #

# === Constants === #

# === Types === #

# === Classes === #

# === Public Functions === #


def project(output: Path) -> None:
    """Place Generated Transport Adapters Behind the C++ Author Surface."""
    client = output / "sdk/cpp/include/dokime/_generated/client.hpp"
    text = client.read_text()
    text = text.replace("struct Client {", "namespace detail {\nstruct Transport {")
    text = text.replace("explicit Client(", "explicit Transport(").replace("static Client connect(", "static Transport connect(")
    text = text.replace("return Client(grpc::", "return Transport(grpc::")
    text = text.replace("} // namespace dokime", "} // namespace detail\n} // namespace dokime")
    text = text.replace('#include "detail/client.hpp"', '#include "detail/client.hpp"\n#include "dokime/errors.hpp"')
    text = text.replace("[reader]", "[reader, context]")
    text = re.sub(
        r'throw std::runtime_error\("(?:Call|Read) ([^:]+): " \+ status.error_message\(\)\);',
        r'throw ::dokime::detail::rpc_error("\1", status, *context);',
        text,
    )
    client.write_text(text, newline="\n")

    messages = output / "sdk/cpp/include/dokime/messages.hpp"
    text = messages.read_text().replace("using ::dokime::detail::payloads::Suite;\n", "")
    messages.write_text(text, newline="\n")

    header = output / "sdk/cpp/include/dokime/_generated/host.hpp"
    text = header.read_text().replace("namespace dokime::plugin {", "namespace dokime::templates {")
    text = text.replace("} // namespace dokime::plugin", "} // namespace dokime::templates")
    header.write_text(text, newline="\n")

    source = output / "sdk/cpp/src/generated/host.cpp"
    text = source.read_text().replace("namespace dokime::plugin {", "namespace dokime::templates {")
    text = text.replace("} // namespace dokime::plugin", "} // namespace dokime::templates")
    text = text.replace(
        "Registrar &Registrar::add(HostOperation operation) {",
        "} // namespace dokime::templates\nnamespace dokime {\n"
        "Registrar &Registrar::add(templates::HostOperation operation) {\n  using templates::HostOperation;\n"
        "  using ::harmos::VISIBILITY_PUBLIC;\n  using ::harmos::plugin::Operation;\n"
        "  using ::harmos::plugin::Dependency;",
    )
    closing = text.rfind("} // namespace dokime::templates")
    text = text[:closing] + text[closing:].replace("} // namespace dokime::templates", "} // namespace dokime", 1)
    source.write_text(text, newline="\n")
    subprocess.run(
        [
            "clang-format",
            "-i",
            "-style={BasedOnStyle: LLVM, ColumnLimit: 140, AllowShortFunctionsOnASingleLine: Empty, AllowShortLambdasOnASingleLine: None}",
            str(client),
            str(messages),
            str(header),
            str(source),
        ],
        check=True,
        creationflags=spawn_flags(),
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    options = parser.parse_args()
    root = Path(__file__).resolve().parents[3]
    settings = configuration(root)
    output = Path(tempfile.mkdtemp(prefix="dokime-cpp-generation-"))
    try:
        sdk.generate(root=output, descriptors=root / "crates/dokime-protocol/src/descriptors.bin", configuration=settings)
        host_resources(output, root / "crates/dokime-protocol/src/descriptors.bin", settings, exclude={"harmos.plugin.v1.Time"})
        project(output)
        expected = {path for path in sdk.inventory(output, settings) if path.parts[:2] == ("sdk", "cpp")}
        existing = {path for path in sdk.inventory(root, settings) if path.parts[:2] == ("sdk", "cpp")}
        differences = [str(path) for path in sorted(expected ^ existing)]
        differences.extend(str(path) for path in sorted(expected & existing) if (root / path).read_bytes() != (output / path).read_bytes())
        if not options.write and differences:
            raise ValueError("Check C++ Bindings: Run mise -C sdk/cpp run generate -- --write\n" + "\n".join(differences))
        if options.write:
            for relative in sorted(existing - expected):
                (root / relative).unlink()
            for relative in sorted(expected):
                destination = root / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes((output / relative).read_bytes())
    finally:
        shutil.rmtree(output)


# === Private Functions === #

# === Entry Point === #

if __name__ == "__main__":
    main()
