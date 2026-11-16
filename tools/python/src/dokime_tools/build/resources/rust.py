# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Generate Rust Resource Surfaces."""

# === Imports === #

# Standard Library Imports
import re
from pathlib import Path

# isort: split

# Third Party Imports
from google.protobuf.descriptor_pb2 import DescriptorProto, FileDescriptorSet

# isort: split

# Local Imports
from . import schema

# Type Checking Imports

# === Public Exports === #

# === Constants === #


# === Types === #

# === Classes === #

# === Public Functions === #


def exports(root: Path, descriptors: FileDescriptorSet) -> None:
    public_names = schema.public_names(descriptors)
    requests = set()
    for requests_method in schema.table(descriptors):
        if requests_method["face"] == "plugin":
            continue
        requests.add(requests_method["request"])

    audiences = schema.payload_audiences(descriptors)
    groups: dict[str, list[str]] = {"shared": [], "client": [], "plugin": []}
    for file in descriptors.file:
        if not file.package.startswith(("dokime.", "harmos.")):
            continue
        if file.name.endswith("guest.proto"):
            continue
        if file.package.startswith("harmos.plugin."):
            continue
        for definition in [*file.message_type, *file.enum_type]:
            full = f"{file.package}.{definition.name}"
            if full in requests:
                continue
            if definition.name in {"Request", "Response"}:
                continue
            if definition.name.startswith("Child"):
                continue
            source = (
                message(full)
                .replace("dokime_protocol::model", "crate::wire::model")
                .replace("dokime_protocol::host", "crate::wire::host")
                .replace("dokime_protocol::plugin", "crate::wire::plugin")
            )
            groups_for_type = (
                {"plugin"}
                if ".plugin." in file.package
                else {"shared"}
                if ".model." in file.package or len(audiences.get(full, set())) > 1
                else audiences.get(full, {"client"})
            )
            for group in groups_for_type:
                alias = schema.public_name(full, public_names)
                exported = source
                if file.package.startswith("harmos."):
                    boundary = "" if group == "shared" else group + "::"
                    exported = "harmos::" + boundary + definition.name
                groups[group].append(f"pub use {exported}" + (f" as {alias}" if alias != definition.name else "") + ";")
            if isinstance(definition, DescriptorProto) and (
                any(not nested.options.map_entry for nested in definition.nested_type)
                or definition.enum_type
                or any(field.HasField("oneof_index") and not field.proto3_optional for field in definition.field)
            ):
                parent = source.rsplit("::", 1)[0]
                for group in groups_for_type:
                    module = parent + "::" + schema.snake(definition.name)
                    if file.package.startswith("harmos."):
                        module = "harmos::" + ("" if group == "shared" else group + "::") + schema.snake(definition.name)
                    groups[group].append(f"pub use {module};")
    for group, lines in groups.items():
        target = root / "sdk/rust/src" / ("exports.rs" if group == "shared" else f"{group}/types.rs")
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(
            "\n".join(["// @generated from protocol descriptors; do not edit.", *sorted(lines), ""]), encoding="utf-8", newline="\n"
        )


def message(full: str) -> str:
    for package, module in [("harmos.host.v1.", "host"), ("harmos.plugin.v1.", "plugin"), ("harmos.model.v1.", "model")]:
        if full.startswith(package):
            return f"harmos_protocol::{module}::" + "::".join(full.removeprefix(package).split("."))
    if full.startswith("google.protobuf."):
        return "prost_types::" + full.rsplit(".", 1)[-1]
    parts = full.split(".")
    version = next(index for index, part in enumerate(parts) if re.fullmatch(r"v[0-9]+", part))
    return schema.snake(parts[0]) + "_protocol::" + "::".join([parts[version - 1], *parts[version + 1 :]])


def call_fields(methods: list[schema.Method]) -> list[schema.Method]:
    """Keep Selectors Positional and Defaultable Read Filters in Call Options."""
    required = {
        "dokime.host.v1.DocumentsGetRequest": {"scope", "document_id"},
        "dokime.host.v1.DocumentsWatchRequest": {"scope", "document_id"},
        "dokime.host.v1.EvidenceStatisticsRequest": {"scope", "stream", "field"},
        "dokime.host.v1.evidence.RecordsListRequest": {"scope", "record_type"},
        "dokime.host.v1.evidence.RecordsWatchRequest": {"scope", "record_type"},
        "dokime.host.v1.evidence.SamplesListRequest": {"scope", "streams"},
        "dokime.host.v1.evidence.SamplesWatchRequest": {"scope", "streams"},
    }
    selected: list[schema.Method] = []
    for method in methods:
        selectors = required.get(method["request"])
        if selectors is not None:
            method = method.copy()
            method["fields"] = [
                schema.Field(**{**field, "optional": field["optional"] or field["name"] not in selectors}) for field in method["fields"]
            ]
        selected.append(method)
    return selected


def render(root: Path, methods: list[schema.Method], *, public_names: dict[str, str], target: Path | None = None) -> None:
    methods = call_fields(methods)
    selected: list[schema.Method] = []
    for method in methods:
        method = method.copy()
        if method["face"] == "client" and method["request"] == "harmos.host.v1.plugins.SettingsListRequest":
            method["fields"] = [
                schema.Field(**{**field, "optional": False}) if field["name"] == "name" else field for field in method["fields"]
            ]
        selected.append(method)
    methods = selected
    nodes = {()}
    for method in methods:
        nodes.update(tuple(method["path"][:length]) for length in range(1, len(method["path"])))
    names = {}
    for names_node in nodes:
        names[names_node] = "Root" if not names_node else "".join(part.title().replace("_", "") for part in names_node)

    used = set()
    for used_method in methods:
        for used_key in ("request", "response"):
            used.add(used_method[used_key])

    used.update(
        field["type"]
        for method in methods
        for field in [*method["fields"], *([method["payload_field"]] if method["payload_field"] else [])]
        if field["type"]
    )
    aliases = {full: schema.public_name(full, public_names) for full in used}
    duplicates = set()
    for duplicates_name in aliases.values():
        if not (list(aliases.values()).count(duplicates_name) > 1):
            continue
        duplicates.add(duplicates_name)

    for full in used:
        if aliases[full] in duplicates:
            aliases[full] = "".join(part.title() for part in full.split(".")[:-1] if not re.fullmatch(r"v[0-9]+", part)) + aliases[full]

    def field_type(field: schema.Field) -> str:
        value = (
            aliases[field["type"]]
            if field["type"]
            else {
                1: "f64",
                2: "f32",
                8: "bool",
                9: "String",
                12: "Vec<u8>",
                4: "u64",
                13: "u32",
                6: "u64",
                7: "u32",
                5: "i32",
                15: "i32",
                17: "i32",
            }.get(field["kind"], "i64")
        )
        return f"std::collections::HashMap<String, {value}>" if field["map"] else f"Vec<{value}>" if field["repeated"] else value

    lines = [
        "// @generated from protocol descriptors; do not edit.",
        "use harmos::client::{CallOptions, ClientTransport, Watching, action_error, request_id};",
        "use prost_reflect::ReflectMessage;",
        "use tonic::Status as RpcStatus;",
        "use tonic::codegen::http::uri::PathAndQuery;",
    ]
    lines_items = []
    for lines_full in sorted(used):
        lines_items.append(
            f"use {message(lines_full)}"
            + (f" as {aliases[lines_full]}" if aliases[lines_full] != lines_full.rsplit(".", 1)[-1] else "")
            + ";"
        )
    lines += lines_items
    for method in methods:
        name = "".join(part.title().replace("_", "") for part in method["path"]) + "Options"
        lines += ["#[derive(Default)]", f"pub struct {name} {{"]
        lines_items = []
        for lines_field in method["fields"]:
            if not (lines_field["optional"]):
                continue
            lines_items.append(f"    pub {lines_field['name']}: Option<{field_type(lines_field)}>,")
        lines += lines_items
        lines += ["    pub context: CallOptions,", "}"]
        lines += [f"impl Clone for {name} {{", "    fn clone(&self) -> Self {", "        Self {"]
        for field in method["fields"]:
            if field["optional"]:
                expression = f"self.{field['name']}" + ("" if field["copy"] else ".clone()")
                lines += [f"            {field['name']}: {expression},"]
        lines += [
            "            context: CallOptions { metadata: self.context.metadata.clone(), timeout: self.context.timeout },",
            "        }",
            "    }",
            "}",
        ]
    for node in sorted(nodes):
        children = sorted(child for child in nodes if len(child) == len(node) + 1 and child[:-1] == node)
        operations = []
        for operations_method in methods:
            if tuple(operations_method["path"][:-1]) != node:
                continue
            operations.append(operations_method)

        lines += ["#[derive(Clone)]", f"pub struct {names[node]} {{"]
        if operations:
            lines += ["    client: ClientTransport,"]
        lines += [f"    pub {child[-1]}: {names[child]}," for child in children]
        lines += ["}", f"impl {names[node]} {{", "    pub fn new(client: ClientTransport) -> Self {", "        Self {"]
        lines += [f"            {child[-1]}: {names[child]}::new(client.clone())," for child in children]
        if operations:
            lines += ["            client,"]
        lines += ["        }", "    }"]
        for method in operations:
            if method["client_streaming"]:
                raise ValueError(f"Generate Rust Client {method['method']}: Managed Contract Required")
            request, response = (aliases[method[key]] for key in ("request", "response"))
            required = []
            for required_field in method["fields"]:
                if required_field["optional"]:
                    continue
                required.append(required_field)

            parameters = []
            for parameters_field in required:
                parameters.append(
                    f"{parameters_field['name']}: "
                    + ("impl Into<String>" if field_type(parameters_field) == "String" else field_type(parameters_field))
                )

            args = [field["name"] for field in required]
            options = "".join(part.title().replace("_", "") for part in method["path"]) + "Options"
            payload = method["payload_field"]
            result = field_type(payload) if payload else response
            if method["server_streaming"]:
                result = f"Watching<{result}>"
            name = method["path"][-1]
            lines += [
                f"    pub async fn {name}(&self{', ' if parameters else ''}{', '.join(parameters)}) -> Result<{result} , RpcStatus> {{",
                f"        self.{name}_with({', '.join([*args, options + '::default()'])}).await",
                "    }",
            ]
            lines += [
                f"    pub async fn {name}_with(&self, {', '.join([*parameters, 'options: ' + options])}) -> Result<{result} , RpcStatus> {{",
                f"        let value = {request} {{",
            ]
            for group in sorted({field["oneof"] for field in method["fields"] if field["oneof"]}):
                members = []
                for members_field in method["fields"]:
                    if members_field["oneof"] != group:
                        continue
                    members.append(members_field)

                variant = message(method["request"]).rsplit("::", 1)[0] + "::" + schema.snake(request) + "::" + group.title()
                variants = ", ".join("options." + field["name"] for field in members)
                lines += [f"            {group}: match ({variants}) {{"]
                for index, member in enumerate(members):
                    pattern = ", ".join("Some(value)" if index == other else "None" for other in range(len(members)))
                    lines += [f"                ({pattern}) => Some({variant}::{member['name'].title()}(value)),"]
                lines += [
                    f'                _ => return Err(RpcStatus::invalid_argument("Choose Exactly One {group} Variant")),',
                    "            },",
                ]
            for field in method["fields"]:
                if field["oneof"]:
                    continue
                name = field["name"]
                if name == "request_id":
                    expression = "match options.request_id { Some(value) => value, None => request_id()?, }"
                elif method["request"] == "harmos.host.v1.plugins.SettingsListRequest" and name == "name":
                    expression = "Some(name.into())"
                elif field["optional"]:
                    expression = f"options.{name}"
                    if not field["wire_optional"] and (field["kind"] != 11 or field["repeated"] or field["map"]):
                        expression += ".unwrap_or_default()"
                elif field["kind"] == 11 and not field["repeated"] and not field["map"]:
                    expression = f"Some({name})"
                elif field_type(field) == "String":
                    expression = f"{name}.into()"
                else:
                    expression = name
                if field["kind"] == 14:
                    expression = f"{expression}.map(|value| value as i32)" if field["optional"] else f"{expression} as i32"
                if (
                    field["wire_optional"]
                    and not field["optional"]
                    and field["kind"] != 11
                    and not (method["request"] == "harmos.host.v1.plugins.SettingsListRequest" and name == "name")
                ):
                    expression = f"Some({expression})"
                lines += [f"            {name}," if expression == name else f"            {name}: {expression},"]
            lines += ["        };", "        let request = options.context.request(value);"]
            path = f"/{method['package']}.{method['service']}/{method['method']}"
            call = "watch_known" if method["server_streaming"] else "unary"
            lines += [
                f'        let response = self.client.{call}::<{request}, {response}>(PathAndQuery::from_static({path!r}), request, {response}::default().descriptor()).await.map_err(|error| action_error("{".".join(method["path"])}", error))?;'.replace(
                    repr(path), '"' + path + '"'
                )
            ]
            if method["server_streaming"]:
                lines += [f'        let response = response.with_action("{".".join(method["path"])}");']
                if payload:
                    field = payload["name"]
                    value = f"response.{field}"
                    if payload["kind"] == 11 and not payload["repeated"]:
                        value += f'.ok_or_else(|| RpcStatus::data_loss("Read {method["service"]}.{method["method"]}: Missing Payload"))'
                    else:
                        value = f"Ok({value})"
                    lines += [f"        Ok(response.map(|response| {value}))"]
                else:
                    lines += ["        Ok(response)"]
            else:
                lines += ["        let response = response.into_inner();"]
                if payload:
                    value = f"response.{payload['name']}"
                    if payload["kind"] == 11 and not payload["repeated"]:
                        lines += [
                            f'        {value}.ok_or_else(|| RpcStatus::data_loss("Read {method["service"]}.{method["method"]}: Missing Payload"))'
                        ]
                    else:
                        lines += [f"        Ok({value})"]
                else:
                    lines += ["        Ok(response)"]
            lines += ["    }"]
        lines += ["}"]
    destination = target or root / "sdk/rust/src/connection/resources.rs"
    destination.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
    if target is None:
        exports = [*names.values(), *("".join(part.title().replace("_", "") for part in method["path"]) + "Options" for method in methods)]
        (root / "sdk/rust/src/client/resources.rs").write_text(
            "// @generated from protocol descriptors; do not edit.\npub use crate::connection::resources::{"
            + ", ".join(sorted(exports))
            + "};\n",
            encoding="utf-8",
            newline="\n",
        )


def host(root: Path, methods: list[schema.Method]) -> None:
    methods = call_fields(methods)

    def title(value: str) -> str:
        return "".join(word.title() for word in value.split("_"))

    def field_type(field: schema.Field) -> str:
        value = (
            field["type"].rsplit(".", 1)[-1]
            if field["type"]
            else {1: "f64", 2: "f32", 8: "bool", 9: "String", 12: "Vec<u8>", 4: "u64", 13: "u32"}.get(field["kind"], "i64")
        )
        return f"std::collections::HashMap<String, {value}>" if field["map"] else f"Vec<{value}>" if field["repeated"] else value

    nodes = sorted({tuple(method["path"][:length]) for method in methods for length in range(1, len(method["path"]))})
    names = {node: "Host" + "".join(title(part) for part in node) + "Resource" for node in nodes}
    used = set()
    for used_method in methods:
        for used_key in ("request", "response"):
            used.add(used_method[used_key])

    used.update(
        field["type"]
        for method in methods
        for field in [*method["fields"], *([method["payload_field"]] if method["payload_field"] else [])]
        if field["type"]
    )
    lines = [
        "// @generated from protocol descriptors; do not edit.",
        "use crate::{Context, Error, Result, Plugin, OperationPolicy};",
        "use std::num::NonZeroUsize;",
    ]
    lines += [f"use {message(full)};" for full in sorted(used)]
    lines += ["pub enum HostOperation {"]
    lines += ["    " + "".join(title(part) for part in method["path"]) + "," for method in methods]
    lines += ["}", "impl Plugin {", "    pub fn requires_host(self, operation: HostOperation) -> Self {", "        match operation {"]
    for method in methods:
        name = "".join(title(part) for part in method["path"])
        lines += [f"            HostOperation::{name} => self.requires::<{method['request'].rsplit('.', 1)[-1]}>(),"]
    lines += [
        "        }",
        "    }",
        "}",
        "impl Context {",
        "    pub fn host(&self) -> HostRoot {",
        "        HostRoot::new(self.clone())",
        "    }",
        "}",
        "#[derive(Clone)]",
        "pub struct HostRoot {",
    ]
    lines_items = []
    for lines_node in nodes:
        if len(lines_node) != 1:
            continue
        lines_items.append(f"    pub {lines_node[-1]}: {names[lines_node]},")
    lines += lines_items
    lines += ["}", "impl HostRoot {", "    fn new(context: Context) -> Self {", "        Self {"]
    lines_items = []
    for lines_node in nodes:
        if len(lines_node) != 1:
            continue
        lines_items.append(f"            {lines_node[-1]}: {names[lines_node]}::new(context.clone()),")
    lines += lines_items
    lines += ["        }", "    }", "}"]
    for method in methods:
        name = "".join(title(part) for part in method["path"])
        lines += ["#[derive(Default)]", f"pub struct Host{name}Options {{"]
        lines_items = []
        for lines_field in method["fields"]:
            if not (lines_field["optional"]):
                continue
            lines_items.append(f"    pub {lines_field['name']}: Option<{field_type(lines_field)}>,")
        lines += lines_items
        lines += (["    pub policy: Option<OperationPolicy>,"] if not method["server_streaming"] else []) + ["}"]
        if method["server_streaming"]:
            q, r = (method[key].rsplit(".", 1)[-1] for key in ("request", "response"))
            payload = method["payload_field"]
            result = field_type(payload) if payload else r
            lines += [
                f"pub struct Host{name} {{",
                f"    incoming: crate::context::Watching<{r}>,",
                "}",
                f"impl Host{name} {{",
                f"    pub async fn next(&mut self) -> Result<Option<{result}>> {{",
                "        let Some(response) = self.incoming.next().await? else { return Ok(None); };",
            ]
            if payload and payload["kind"] == 11 and not payload["repeated"]:
                lines += [
                    f'        response.{payload["name"]}.map(Some).ok_or_else(|| Error::Invalid("Read {name}: Missing Payload".into()))'
                ]
            else:
                lines += [f"        Ok(Some(response{'.' + payload['name'] if payload else ''}))"]
            lines += ["    }", "}"]
    for node in nodes:
        lines += [
            "#[derive(Clone)]",
            f"pub struct {names[node]} {{",
            *(["    context: Context,"] if any(tuple(method["path"][:-1]) == node for method in methods) else []),
            *[f"    pub {child[-1]}: {names[child]}," for child in nodes if child[:-1] == node],
            "}",
            f"impl {names[node]} {{",
            "    fn new(context: Context) -> Self { Self {",
            *[f"        {child[-1]}: {names[child]}::new(context.clone())," for child in nodes if child[:-1] == node],
            *(["        context,"] if any(tuple(method["path"][:-1]) == node for method in methods) else []),
            "    } }",
        ]
        for method in (method for method in methods if tuple(method["path"][:-1]) == node):
            name = "".join(title(part) for part in method["path"])
            q, r = (method[key].rsplit(".", 1)[-1] for key in ("request", "response"))
            required = []
            for required_field in method["fields"]:
                if required_field["optional"]:
                    continue
                required.append(required_field)

            parameters = [f"{field['name']}: {field_type(field)}" for field in required]
            args = [field["name"] for field in required]
            if method["server_streaming"]:
                parameters += ["max_item_bytes: NonZeroUsize"]
                args += ["max_item_bytes"]
            payload = method["payload_field"]
            result = f"Host{name}" if method["server_streaming"] else field_type(payload) if payload else r
            action = method["path"][-1]
            lines += [
                f"    pub async fn {action}(&self{', ' if parameters else ''}{', '.join(parameters)}) -> Result<{result}> {{",
                f"        self.{action}_with({', '.join([*args, 'Host' + name + 'Options::default()'])}).await",
                "    }",
                f"    pub async fn {action}_with(&self, {', '.join([*parameters, 'options: Host' + name + 'Options'])}) -> Result<{result}> {{",
                *(
                    [f"        let Host{name}Options {{}} = options;"]
                    if method["server_streaming"] and not any(field["optional"] for field in method["fields"])
                    else []
                ),
                f"        let request = {q} {{",
            ]
            for group in sorted({field["oneof"] for field in method["fields"] if field["oneof"]}):
                members = []
                for members_field in method["fields"]:
                    if members_field["oneof"] != group:
                        continue
                    members.append(members_field)

                variant = (
                    message(method["request"]).rsplit("::", 1)[0]
                    + "::"
                    + schema.snake(method["request"].rsplit(".", 1)[-1])
                    + "::"
                    + title(group)
                )
                selected = ", ".join("options." + field["name"] for field in members)
                lines += [f"            {group}: match ({selected}) {{"]
                for index, member in enumerate(members):
                    pattern = ", ".join("Some(value)" if index == other else "None" for other in range(len(members)))
                    lines += [f"                ({pattern}) => Some({variant}::{title(member['name'])}(value)),"]
                lines += [
                    f'                _ => return Err(Error::Invalid("Choose Exactly One {group} Variant".into())),',
                    "            },",
                ]
            for field in method["fields"]:
                if field["oneof"]:
                    continue
                value = (
                    f"options.{field['name']}"
                    if field["optional"]
                    else f"Some({field['name']})"
                    if field["kind"] == 11 and not field["map"] and not field["repeated"]
                    else field["name"]
                )
                if field["name"] == "request_id":
                    value = "match options.request_id { Some(value) => value, None => harmos::client::request_id()?, }"
                elif field["optional"] and not field["wire_optional"] and (field["kind"] != 11 or field["repeated"] or field["map"]):
                    value += ".unwrap_or_default()"
                if field["kind"] == 14:
                    value = f"{value}.map(|value| value as i32)" if field["optional"] else f"{value} as i32"
                lines += [f"            {field['name']}," if value == field["name"] else f"            {field['name']}: {value},"]
            lines += ["        };"]
            if method["server_streaming"]:
                lines += [
                    f"        Ok(Host{name} {{ incoming: self.context.watch(request, max_item_bytes).await? }})",
                ]
            else:
                lines += [f"        let response: {r} = self.context.call_with(request, options.policy).await?;"]
                if payload and payload["kind"] == 11 and not payload["repeated"]:
                    lines += [f'        response.{payload["name"]}.ok_or_else(|| Error::Invalid("Read {name}: Missing Payload".into()))']
                else:
                    lines += [f"        Ok(response{'.' + payload['name'] if payload else ''})"]
            lines += ["    }"]
        lines += ["}"]
    target = root / "sdk/rust/src/plugin/host.rs"
    target.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
    exports = ["HostOperation", "HostRoot", *names.values()]
    for method in methods:
        name = "".join(title(part) for part in method["path"])
        exports += [f"Host{name}Options"]
        if method["server_streaming"]:
            exports += [f"Host{name}"]
    (target.parent / "host_exports.rs").write_text(
        "// @generated from protocol descriptors; do not edit.\npub use host::{" + ", ".join(sorted(exports)) + "};\n",
        encoding="utf-8",
        newline="\n",
    )


# === Private Functions === #

# === Entry Point === #
