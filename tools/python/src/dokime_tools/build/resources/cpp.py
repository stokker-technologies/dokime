# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Generate Cpp Resource Surfaces."""

# === Imports === #

# Standard Library Imports
import re
import subprocess
from pathlib import Path

# isort: split

# Third Party Imports
from google.protobuf.descriptor_pb2 import FileDescriptorSet

# isort: split

# Local Imports
from ...process import spawn_flags
from . import schema

# Type Checking Imports

# === Public Exports === #

# === Constants === #


# === Types === #

# === Classes === #

# === Public Functions === #


def exports(root: Path, descriptors: FileDescriptorSet) -> dict[str, str]:
    public_names = schema.public_names(descriptors)
    type_names: dict[str, str] = {}
    methods = schema.table(descriptors)
    audiences = schema.payload_audiences(descriptors)
    requests = set()
    for requests_method in methods:
        if requests_method["face"] == "plugin":
            continue
        requests.add(requests_method["request"])

    for file in descriptors.file:
        if not file.package.startswith("harmos."):
            continue
        for definition in [*file.message_type, *file.enum_type]:
            full = f"{file.package}.{definition.name}"
            faces = audiences.get(full, {"client"})
            boundary = "" if ".model." in file.package or len(faces) > 1 else next(iter(faces))
            type_names[full] = "::harmos::" + (boundary + "::" if boundary else "") + definition.name
    carriers = {
        "Request",
        "Response",
        "ChildRequest",
        "ChildResponse",
        "ChildCancel",
        "ChildWatch",
        "ChildWatchResponse",
        "ChildWatchCancel",
    }
    private = ["// @generated from protocol descriptors; do not edit.", "#pragma once"]
    public = ["// @generated from protocol descriptors; do not edit.", "#pragma once", '#include "detail/messages.hpp"']
    entries = []
    hidden = []
    for file in descriptors.file:
        if file.name.endswith("guest.proto"):
            continue
        if not file.package.startswith("dokime."):
            continue
        private += [f'#include "{file.name.removesuffix(".proto")}.pb.h"']
        boundary = "client" if ".host." in file.package else ""
        for definition in [*file.message_type, *file.enum_type]:
            groups_for_type = {"plugin"} if ".plugin." in file.package else audiences.get(f"{file.package}.{definition.name}", {"client"})
            boundary = "" if ".model." in file.package or len(groups_for_type) > 1 else next(iter(groups_for_type))
            if f"{file.package}.{definition.name}" in requests or definition.name in carriers:
                alias = (boundary.title() or "Shared") + schema.public_name(f"{file.package}.{definition.name}", public_names)
                hidden.append(f"using {alias} = ::{file.package.replace(chr(46), chr(58) + chr(58))}::{definition.name};")
                continue
            name = "FailureValue" if definition.name == "Failure" else schema.public_name(f"{file.package}.{definition.name}", public_names)
            type_names[f"{file.package}.{definition.name}"] = "::dokime::" + (boundary + "::" if boundary else "") + name
            alias = (boundary.title() or "Shared") + name
            entries.append((boundary, name, alias, f"::{file.package.replace('.', '::')}::{definition.name}", False))
        for definition in file.enum_type:
            for value in definition.value:
                prefix = schema.snake(definition.name).upper() + "_"
                name = value.name if value.name.startswith(prefix) else prefix + value.name
                entries.append(
                    (
                        boundary,
                        name,
                        (boundary.title() or "Shared") + name,
                        f"static_cast<::{file.package.replace('.', '::')}::{definition.name}>({value.number})",
                        True,
                    )
                )
    private += [
        "namespace dokime::detail {",
        'inline constexpr char stream_fingerprint_prefix[] = "harmos.protobuf.stream.v1\\0";',
        *hidden,
    ]
    private_items = []
    for private__, private__, private_alias, private_full, private_constant in entries:
        private_items.append(
            f"inline constexpr auto {private_alias} = {private_full};" if private_constant else f"using {private_alias} = {private_full};"
        )
    private += private_items
    private += ["}"]
    for boundary in ("", "client", "plugin"):
        public += ["namespace dokime" + (f"::{boundary}" if boundary else "") + " {"]
        public_items = []
        for public_face, public_name, public_alias, public__, public_constant in entries:
            if public_face != boundary:
                continue
            public_items.append(
                f"inline constexpr auto {public_name} = ::dokime::detail::{public_alias};"
                if public_constant
                else f"using {public_name} = ::dokime::detail::{public_alias};"
            )
        public += public_items
        public += ["}"]
    destination = root / "sdk/cpp/include/dokime"
    (destination / "detail").mkdir(parents=True, exist_ok=True)
    (destination / "detail/messages.hpp").write_text("\n".join(private) + "\n", encoding="utf-8", newline="\n")
    (destination / "messages.hpp").write_text("\n".join(public) + "\n", encoding="utf-8", newline="\n")
    format([destination / "messages.hpp", destination / "detail/messages.hpp"])

    return type_names


def host(root: Path, methods: list[schema.Method], type_names: dict[str, str]) -> None:
    def title(value: str) -> str:
        return "".join(word.title() for word in value.split("_"))

    def field_type(field: schema.Field) -> str:
        value = (
            message(field["type"], type_names)
            if field["type"]
            else {8: "bool", 9: "std::string", 12: "std::string", 4: "std::uint64_t", 13: "std::uint32_t"}.get(
                field["kind"], "std::int64_t"
            )
        )
        return f"std::map<std::string, {value}>" if field["map"] else f"std::vector<{value}>" if field["repeated"] else value

    nodes = sorted({tuple(method["path"][:length]) for method in methods for length in range(1, len(method["path"]))})
    names = {node: "Host" + "".join(title(part) for part in node) + "Resource" for node in nodes}
    header = [
        "// @generated from protocol descriptors; do not edit.",
        "#pragma once",
        "namespace dokime::plugin {",
        "enum class HostOperation {",
    ]
    header += ["  " + "".join(title(part) for part in method["path"]) + "," for method in methods]
    header += ["};"]
    source = ["// @generated from protocol descriptors; do not edit.", '#include "dokime/dokime.hpp"']
    source_items = []
    for source_file in sorted({method["file"] for method in methods}):
        source_items.append(f'''#include "{source_file.removesuffix(".proto")}.pb.h"''')
    source += source_items
    used = sorted({method[key] for method in methods for key in ("request", "response")})
    # Source Files Import Named Wire Types Once; Headers Mention Only Public Types.
    source += [f"using ::{full.replace('.', '::')};" for full in used]
    source += [
        "namespace dokime::plugin {",
        "using ::harmos::plugin::Operation;",
        "using ::harmos::plugin::Watching;",
        "using ::harmos::VISIBILITY_PUBLIC;",
    ]
    for method in methods:
        name = "".join(title(part) for part in method["path"])
        header += [f"struct Host{name}Options {{"]
        header_items = []
        for header_field in method["fields"]:
            if not (header_field["optional"]):
                continue
            header_items.append(f"  std::optional<{field_type(header_field)}> {header_field['name']};")
        header += header_items
        if not method["server_streaming"]:
            header += ["  std::optional<::harmos::OperationPolicy> policy;"]
        header += ["};"]
        if method["server_streaming"]:
            payload = method["payload_field"]
            value = field_type(payload) if payload else message(method["response"], type_names)
            header += [
                f"class Host{name} {{",
                "public:",
                f"  explicit Host{name}(std::function<std::optional<{value}>()> next) : next_(std::move(next)) {{}}",
                f"  std::optional<{value}> next() {{ return next_(); }}",
                "private:",
                f"  std::function<std::optional<{value}>()> next_;",
                "};",
            ]
    for node in reversed(nodes):
        children = []
        for children_child in nodes:
            if children_child[:-1] != node:
                continue
            children.append(children_child)

        header += [f"struct {names[node]} {{", "  Context context;"]
        header += [f"  {names[child]} {child[-1]};" for child in children]
        initializers = ["context(value)", *[f"{child[-1]}(value)" for child in children]]
        header += [f"  {names[node]}(Context value) : {', '.join(initializers)} {{}}"]
        for method in (method for method in methods if tuple(method["path"][:-1]) == node):
            name = "".join(title(part) for part in method["path"])
            q, r = (method[key].rsplit(".", 1)[-1] for key in ("request", "response"))
            payload = method["payload_field"]
            result = (
                f"Host{name}" if method["server_streaming"] else field_type(payload) if payload else message(method["response"], type_names)
            )
            parameters = []
            for parameters_field in method["fields"]:
                if parameters_field["optional"]:
                    continue
                parameters.append(f"const {field_type(parameters_field)}& {parameters_field['name']}")

            if method["server_streaming"]:
                parameters += ["std::size_t max_item_bytes"]
            header += [
                f"  {result} {method['path'][-1] + ('_' if method['path'][-1] in {'delete', 'export', 'import', 'new', 'class', 'default'} else '')}({', '.join([*parameters, 'Host' + name + 'Options options = {}'])});"
            ]
            source += [
                f"{result} {names[node]}::{method['path'][-1] + ('_' if method['path'][-1] in {'delete', 'export', 'import', 'new', 'class', 'default'} else '')}({', '.join([*parameters, 'Host' + name + 'Options options'])}) {{",
                f"  {q} request;",
            ]
            for field in method["fields"]:
                if field["name"] == "request_id":
                    source += ["  request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());"]
                    continue
                value = "*options." + field["name"] if field["optional"] else field["name"]
                if field["optional"]:
                    source += [f"  if (options.{field['name']}) {{"]
                assignment = (
                    f"for (const auto& [key, item] : {value}) {{ (*request.mutable_{field['name']}())[key] = item; }}"
                    if field["map"]
                    else f"request.mutable_{field['name']}()->Assign({value}.begin(), {value}.end());"
                    if field["repeated"]
                    else f"*request.mutable_{field['name']}() = {value};"
                    if field["kind"] == 11
                    else f"request.set_{field['name']}({value});"
                )
                source += ["    " + assignment]
                if field["optional"]:
                    source += ["  }"]
            source += [
                f'  auto operation = Operation<{q}, {r}>("{method["package"]}.{method["service"]}/{method["method"]}", VISIBILITY_PUBLIC);'
            ]
            if method["server_streaming"]:
                source += [
                    "  (void)options;",
                    "  operation.watch();",
                    f"  auto incoming = std::make_shared<Watching<{r}>>(context.watch(operation, request, max_item_bytes));",
                    f"  return Host{name}([incoming]() -> std::optional<{field_type(payload) if payload else message(method['response'], type_names)}> {{",
                    "    auto response = incoming->next();",
                    "    if (!response) { return {}; }",
                ]
                value = (
                    f"std::move(*response->mutable_{payload['name']}())"
                    if payload and payload["kind"] == 11
                    else f"response->{payload['name']}()"
                    if payload
                    else "*response"
                )
                if payload and payload["kind"] == 11 and not payload["repeated"]:
                    source += [
                        f'    if (!response->has_{payload["name"]}()) {{ throw std::runtime_error("Read {name}: Missing Payload"); }}'
                    ]
                source += [f"    return {value};", "  });"]
            else:
                source += ["  auto response = context.call(operation, request, options.policy).get();"]
                if payload and payload["repeated"]:
                    value = f"{field_type(payload)}(response.{payload['name']}().begin(), response.{payload['name']}().end())"
                elif payload and payload["kind"] == 11 and not payload["repeated"]:
                    source += [f'  if (!response.has_{payload["name"]}()) {{ throw std::runtime_error("Read {name}: Missing Payload"); }}']
                    value = f"std::move(*response.mutable_{payload['name']}())"
                else:
                    value = f"response.{payload['name']}()" if payload else "response"
                source += [f"  return {value};"]
            source += ["}"]
        header += ["};"]
    header += ["struct HostRoot {"]
    header_items = []
    for header_node in nodes:
        if len(header_node) != 1:
            continue
        header_items.append(f"  {names[header_node]} {header_node[-1]};")
    header += header_items
    header += ["};", "}"]
    source += [
        "}",
        "namespace dokime::plugin {",
        "HostRoot Context::host() {",
        "  return {" + ", ".join("{*this}" for node in nodes if len(node) == 1) + "};",
        "}",
        "Plugin& Plugin::requires_operation(HostOperation operation) {",
        "  switch (operation) {",
    ]
    for method in methods:
        name = "".join(title(part) for part in method["path"])
        q, r = (method[key].rsplit(".", 1)[-1] for key in ("request", "response"))
        expression = f'Operation<{q}, {r}>("{method["package"]}.{method["service"]}/{method["method"]}", VISIBILITY_PUBLIC)' + (
            ".watch()" if method["server_streaming"] else ""
        )
        source += [f"    case HostOperation::{name}: return requires_operation({expression});"]
    source += ['    default: throw std::invalid_argument("Declare Host Requirement: Unknown Operation");', "  }", "}", "}"]
    boundary = source.index("namespace dokime::plugin {")
    body = "\n".join(source[boundary:])
    named = set()
    for named_method in methods:
        if named_method["payload_field"] is not None:
            continue
        named.add(message(named_method["response"], type_names))

    named.update(
        message(field["type"], type_names)
        for method in methods
        for field in [*method["fields"], *([method["payload_field"]] if method["payload_field"] else [])]
        if field["type"]
    )
    named.update(
        f"std::{name}"
        for name in (
            "string",
            "vector",
            "optional",
            "size_t",
            "uint64_t",
            "uint32_t",
            "int64_t",
            "int32_t",
            "runtime_error",
            "invalid_argument",
        )
    )
    imported = set()
    for imported_method in methods:
        for imported_key in ("request", "response"):
            imported.add(imported_method[imported_key].rsplit(".", 1)[-1])

    declarations = []
    for qualified in sorted(named, key=len, reverse=True):
        if qualified not in body:
            continue
        name = qualified.rsplit("::", 1)[-1]
        body = re.sub(re.escape(qualified) + r"\b", name, body)
        if name not in imported:
            declarations.append(f"using {qualified};")
            imported.add(name)
    source = [*source[:boundary], *sorted(declarations), body]
    folder = root / "sdk/cpp/include/dokime/_generated"
    folder.mkdir(parents=True, exist_ok=True)
    (folder / "host.hpp").write_text("\n".join(header) + "\n", encoding="utf-8", newline="\n")
    folder = root / "sdk/cpp/src/generated"
    folder.mkdir(parents=True, exist_ok=True)
    (folder / "host.cpp").write_text("\n".join(source) + "\n", encoding="utf-8", newline="\n")
    format([root / "sdk/cpp/include/dokime/_generated/host.hpp", folder / "host.cpp"])


def message(full: str, type_names: dict[str, str]) -> str:
    parts = full.split(".")
    if full.startswith("google.protobuf."):
        return "::google::protobuf::" + parts[-1]
    if full.startswith(("harmos.host.", "dokime.host.")):
        # Shared Host Replies Belong to the Root; Audience-Exclusive Replies to Their Barrel.
        return type_names.get(full, "::" + parts[0] + "::client::" + parts[-1])
    if full.startswith(("harmos.model.", "dokime.model.")):
        return type_names.get(full, "::" + parts[0] + "::" + parts[-1])
    return type_names.get(full, "::" + parts[0] + "::plugin::" + parts[-1])


def render(
    root: Path, methods: list[schema.Method], *, type_names: dict[str, str], namespace: str = "harmos", target: Path | None = None
) -> None:
    library = root / "crates/dokime-protocol/src/descriptors.bin"
    if library.is_file():
        descriptors = FileDescriptorSet.FromString(library.read_bytes())
        for full, groups in schema.payload_audiences(descriptors).items():
            if not full.startswith("harmos.host."):
                continue
            boundary = "" if len(groups) > 1 else next(iter(groups)) + "::"
            type_names[full] = "::harmos::" + boundary + full.rsplit(".", 1)[-1]
    nodes = {()}
    for method in methods:
        nodes.update(tuple(method["path"][:length]) for length in range(1, len(method["path"])))

    def title(part: str) -> str:
        return "".join(word.title() for word in part.split("_"))

    def field_type(field: schema.Field) -> str:
        value = (
            message(field["type"], type_names)
            if field["type"]
            else {
                1: "double",
                2: "float",
                8: "bool",
                9: "std::string",
                12: "std::string",
                4: "std::uint64_t",
                13: "std::uint32_t",
                6: "std::uint64_t",
                7: "std::uint32_t",
                5: "std::int32_t",
                15: "std::int32_t",
                17: "std::int32_t",
            }.get(field["kind"], "std::int64_t")
        )
        return f"std::map<std::string, {value}>" if field["map"] else f"std::vector<{value}>" if field["repeated"] else value

    names = {}
    for names_node in nodes:
        names[names_node] = "Client" if not names_node else "".join(title(part) for part in names_node) + "Resource"

    lines = [
        "// @generated from protocol descriptors; do not edit.",
        "#pragma once",
        '#include "harmos/client_transport.hpp"',
        '#include "harmos/messages.hpp"',
        '#include "dokime/messages.hpp"',
    ]
    # Wire Stubs and Request Types Stay in a Private Generated Header.
    private = ["// @generated from protocol descriptors; do not edit.", "#pragma once"]
    includes = sorted({method["file"].removesuffix(".proto") + ".grpc.pb.h" for method in methods})
    private += [f'#include "{include}"' for include in includes]
    private += [f"namespace {namespace.replace('.', '::')}::detail {{"]
    for method in methods:
        prefix = "".join(title(part) for part in method["path"])
        private += [
            f"using {prefix}Request = ::{method['request'].replace('.', '::')};",
            f"using {prefix}Response = ::{method['response'].replace('.', '::')};",
            f"using {prefix}Service = ::{method['package'].replace('.', '::')}::{method['service']};",
        ]
    private += ["}"]
    destination = target or root / "sdk/cpp/include/harmos/_generated/client.hpp"
    destination.parent.mkdir(parents=True, exist_ok=True)
    wire = destination.parent / "detail/client.hpp"
    wire.parent.mkdir(parents=True, exist_ok=True)
    wire.write_text("\n".join(private) + "\n", encoding="utf-8", newline="\n")
    lines += ['#include "detail/client.hpp"']
    if namespace not in {"harmos", "dokime"}:
        public = ["// @generated from protocol descriptors; do not edit.", "#pragma once", '#include "detail/client.hpp"']
        for method in methods:
            if method["owner"] == "harmos":
                continue
            prefix = "".join(title(part) for part in method["path"])
            public += [
                f"namespace {method['owner'].replace('.', '::')}::client {{ using {method['response'].rsplit('.', 1)[-1]} = ::{namespace.replace('.', '::')}::detail::{prefix}Response; }}"
            ]
        (destination.parent / "client_messages.hpp").write_text("\n".join(public) + "\n", encoding="utf-8", newline="\n")
        lines += ['#include "client_messages.hpp"']
    lines += [f"namespace {namespace.replace('.', '::')} {{"]
    lines += ["namespace client {"]
    for method in methods:
        options = "".join(title(part) for part in method["path"]) + "Options"
        lines += [f"struct {options} {{"]
        lines_items = []
        for lines_field in method["fields"]:
            if not (lines_field["optional"]):
                continue
            lines_items.append(f"  std::optional<{field_type(lines_field)}> {lines_field['name']};")
        lines += lines_items
        lines += ["  std::shared_ptr<grpc::ClientContext> context;", "};"]
    lines += ["}"]

    def sort_key(node: tuple[str, ...]) -> tuple:
        return (-len(node), node)

    for node in sorted(nodes, key=sort_key):
        children = sorted(child for child in nodes if len(child) == len(node) + 1 and child[:-1] == node)
        operations = []
        for operations_method in methods:
            if tuple(operations_method["path"][:-1]) != node:
                continue
            operations.append(operations_method)

        lines += [f"struct {names[node]} {{"]
        if operations:
            lines += ["  std::shared_ptr<grpc::Channel> channel;"]
        lines += [f"  {names[child]} {child[-1]};" for child in children]
        initializers = (["channel(channel)"] if operations else []) + [f"{child[-1]}(channel)" for child in children]
        lines += [f"  explicit {names[node]}(std::shared_ptr<grpc::Channel> channel) : {', '.join(initializers)} {{}}"]
        if not node:
            lines += [
                "  static Client connect(std::string address, const ::harmos::StorageTransferSettings& transfer = {}, std::optional<int> message_bytes = {}) {",
                "    const auto maximum = ::harmos::transfer_message_bytes(transfer, message_bytes);",
                "    grpc::ChannelArguments arguments;",
                "    arguments.SetMaxSendMessageSize(maximum);",
                "    arguments.SetMaxReceiveMessageSize(maximum);",
                '    if (address.starts_with("http://")) {',
                "      address.erase(0, 7);",
                "    }",
                "    return Client(grpc::CreateCustomChannel(address, grpc::InsecureChannelCredentials(), arguments));",
                "  }",
            ]
        for method in operations:
            if method["client_streaming"]:
                raise ValueError(f"Generate C++ Client {method['method']}: Managed Contract Required")
            prefix = "".join(title(part) for part in method["path"])
            request, response, service = (f"detail::{prefix}{suffix}" for suffix in ("Request", "Response", "Service"))
            parameters = []
            for parameters_field in method["fields"]:
                if parameters_field["optional"]:
                    continue
                parameters.append(f"const {field_type(parameters_field)}& {parameters_field['name']}")

            parameters += [f"client::{prefix}Options options = {{}}"]
            payload = method["payload_field"]
            result = field_type(payload) if payload else message(method["response"], type_names)
            stream = method["server_streaming"]
            returns = f"::harmos::ClientStream<{result}>" if stream else result
            name = method["path"][-1]
            if name in {"delete", "export", "import", "new", "class", "default"}:
                name += "_"
            lines += [f"  {returns} {name}({', '.join(parameters)}) {{", f"    {request} request;"]
            for field in method["fields"]:
                name = field["name"]
                optional = field["optional"]
                if name == "request_id":
                    lines += ["    request.set_request_id(options.request_id ? *options.request_id : ::harmos::detail::request_id());"]
                    continue
                if optional:
                    lines += [f"    if (options.{name}) {{"]
                value = f"*options.{name}" if optional else name
                indent = "      " if optional else "    "
                if field["map"]:
                    lines += [
                        f"{indent}for (const auto& [key, item] : {value}) {{",
                        f"{indent}  (*request.mutable_{name}())[key] = item;",
                        f"{indent}}}",
                    ]
                elif field["repeated"]:
                    lines += [f"{indent}request.mutable_{name}()->Assign({value}.begin(), {value}.end());"]
                elif field["kind"] == 11:
                    lines += [f"{indent}*request.mutable_{name}() = {value};"]
                else:
                    lines += [f"{indent}request.set_{name}({value});"]
                if optional:
                    lines += ["    }"]
            lines += ["    auto context = options.context ? options.context : std::make_shared<grpc::ClientContext>();"]
            action = ".".join(method["path"])
            if stream:
                lines += [
                    f"    auto reader = std::shared_ptr<grpc::ClientReader<{response}>>({service}::NewStub(channel)->{method['method']}(context.get(), request));",
                    f"    return {{context, [reader]() -> std::optional<{result}> {{",
                    f"      {response} response;",
                    "      if (!reader->Read(&response)) {",
                    "        const auto status = reader->Finish();",
                    f'        if (!status.ok()) {{ throw std::runtime_error("Read {action}: " + status.error_message()); }}',
                    "        return {};",
                    "      }",
                ]
                indent = "      "
            else:
                lines += [
                    f"    {response} response;",
                    f"    const auto status = {service}::NewStub(channel)->{method['method']}(context.get(), request, &response);",
                    "    if (!status.ok()) {",
                    f'      throw std::runtime_error("Call {action}: " + status.error_message());',
                    "    }",
                ]
                indent = "    "
            value = "response"
            if payload:
                name = payload["name"]
                if payload["repeated"]:
                    value = f"{result}(response.{name}().begin(), response.{name}().end())"
                elif payload["kind"] == 11:
                    lines += [
                        f"{indent}if (!response.has_{name}()) {{",
                        f'{indent}  throw std::runtime_error("Read {action}: Missing Payload");',
                        f"{indent}}}",
                    ]
                    value = f"std::move(*response.mutable_{name}())"
                else:
                    value = f"response.{name}()"
            lines += [f"{indent}return {value};"]
            if stream:
                lines += ["    }};"]
            lines += ["  }"]
        lines += ["};"]
    lines += ["}"]
    destination.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
    format([destination, wire, *([destination.parent / "client_messages.hpp"] if namespace not in {"harmos", "dokime"} else [])])


def format(paths: list[Path]) -> None:
    policy = "{BasedOnStyle: LLVM, ColumnLimit: 140}"
    subprocess.run(
        [
            "mise",
            "-C",
            str(Path(__file__).resolve().parents[6] / "sdk/cpp"),
            "exec",
            "--",
            "clang-format",
            f"--style={policy}",
            "-i",
            *map(str, paths),
        ],
        check=True,
        creationflags=spawn_flags(),
    )


# === Private Functions === #

# === Entry Point === #
