#!/usr/bin/env python3
"""Regenerate `proto/localharness.proto` from an upstream wheel.

The harness schema is not published as a `.proto` file — it ships only as a
serialized `FileDescriptorProto` embedded in the generated
`google/antigravity/proto/localharness_pb2.py`. This script decodes that
descriptor and renders readable `.proto` text, so the schema is generated rather
than hand-transcribed. Hand-transcription is how the crate ended up with
`UsageMetadata` counters as `int32` where the harness declares `uint64`.

Usage:
    pip install protobuf
    # unpack any google-antigravity wheel (they are plain zip files)
    python3 scripts/gen_proto.py /path/to/unpacked-wheel > proto/localharness.proto

Two deliberate divergences from the upstream descriptor, both required:

1. **`syntax = "proto3"`, not editions.** The 0.1.9 descriptor reports
   `syntax='editions'` / `edition=1001`; prost-build 0.12 cannot parse editions.

2. **Every singular field is marked `optional`.** Editions give explicit presence
   by default, and the rendered text would otherwise lose it. The crate depends
   on presence throughout — the step-type classifier and the request dedup in
   `src/local.rs` are built on `.is_some()` — so dropping it would silently turn
   "absent" into "present and zero".

`content.proto` (package `genai`) is deliberately NOT vendored: the only fields
referencing it are `ToolCall.arguments` and `ToolResponse.response`, which
neither SDK populates (upstream reads `arguments_json`). They are skipped, and
`build.rs` sets `ignore_unknown_fields` so a stray value is ignored rather than
failing the whole event.
"""

from __future__ import annotations

import ast
import os
import sys

from google.protobuf import descriptor_pb2

SCALARS = {
    1: "double", 2: "float", 3: "int64", 4: "uint64", 5: "int32",
    6: "fixed64", 7: "fixed32", 8: "bool", 9: "string", 10: "group",
    12: "bytes", 13: "uint32", 15: "sfixed32", 16: "sfixed64",
    17: "sint32", 18: "sint64",
}

LABEL_REPEATED = 3

# Fields whose type lives in content.proto (package `genai`). See the module
# docstring: not vendored, and unpopulated by either SDK.
SKIPPED_TYPE_PREFIXES = (".genai.",)


def load_descriptor(pb2_path: str) -> descriptor_pb2.FileDescriptorProto:
    """Pulls the serialized FileDescriptorProto out of a generated _pb2.py."""
    tree = ast.parse(open(pb2_path, "rb").read().decode("utf-8", "replace"))
    for node in ast.walk(tree):
        if isinstance(node, ast.Call) and getattr(node.func, "attr", "") == "AddSerializedFile":
            fd = descriptor_pb2.FileDescriptorProto()
            fd.ParseFromString(ast.literal_eval(node.args[0]))
            return fd
    raise SystemExit(f"no AddSerializedFile call found in {pb2_path}")


def collect_map_entries(msg: descriptor_pb2.DescriptorProto, prefix: str, out: dict) -> None:
    """Records nested map-entry types so map fields render as `map<K, V>`."""
    for nested in msg.nested_type:
        full = f"{prefix}.{nested.name}"
        if nested.options.map_entry:
            key = next(f for f in nested.field if f.name == "key")
            value = next(f for f in nested.field if f.name == "value")
            out[full] = (type_name(key, {}), type_name(value, {}))
        collect_map_entries(nested, full, out)


def type_name(field: descriptor_pb2.FieldDescriptorProto, maps: dict) -> str:
    if field.type in (11, 14):  # message, enum
        return field.type_name.lstrip(".").split(".")[-1]
    return SCALARS.get(field.type, f"UNKNOWN_{field.type}")


def render_field(field, maps: dict, indent: str, oneof: bool) -> str | None:
    if any(field.type_name.startswith(p) for p in SKIPPED_TYPE_PREFIXES):
        return f"{indent}// omitted: {field.name} = {field.number} " \
               f"({field.type_name}, defined in content.proto — see scripts/gen_proto.py)"

    key = field.type_name.lstrip(".")
    map_kv = maps.get(key)
    if map_kv and field.label == LABEL_REPEATED:
        k, v = map_kv
        return f"{indent}map<{k}, {v}> {field.name} = {field.number};"

    if field.label == LABEL_REPEATED:
        return f"{indent}repeated {type_name(field, maps)} {field.name} = {field.number};"

    # Singular: always explicit-presence. See the module docstring.
    prefix = "" if oneof else "optional "
    return f"{indent}{prefix}{type_name(field, maps)} {field.name} = {field.number};"


def render_enum(enum, indent: str, out: list) -> None:
    out.append(f"{indent}enum {enum.name} {{")
    for value in enum.value:
        out.append(f"{indent}  {value.name} = {value.number};")
    out.append(f"{indent}}}")


def render_message(msg, maps: dict, prefix: str, indent: str, out: list) -> None:
    full = f"{prefix}.{msg.name}" if prefix else msg.name
    out.append(f"{indent}message {msg.name} {{")

    grouped: dict[int, list] = {}
    for field in msg.field:
        in_oneof = field.HasField("oneof_index") and not field.proto3_optional
        line = render_field(field, maps, indent + ("    " if in_oneof else "  "), in_oneof)
        if line is None:
            continue
        if in_oneof:
            grouped.setdefault(field.oneof_index, []).append(line)
        else:
            out.append(line)

    for index, oneof in enumerate(msg.oneof_decl):
        if index in grouped:
            out.append(f"{indent}  oneof {oneof.name} {{")
            out.extend(grouped[index])
            out.append(f"{indent}  }}")

    for enum in msg.enum_type:
        render_enum(enum, indent + "  ", out)

    for nested in msg.nested_type:
        if nested.options.map_entry:
            continue  # rendered inline as map<K, V>
        render_message(nested, maps, full, indent + "  ", out)

    out.append(f"{indent}}}")


def main() -> None:
    if len(sys.argv) < 2:
        raise SystemExit(__doc__)
    root = os.path.abspath(sys.argv[1])
    pb2 = os.path.join(root, "google", "antigravity", "proto", "localharness_pb2.py")
    if not os.path.exists(pb2):  # pre-0.1.8 layout
        pb2 = os.path.join(root, "google", "antigravity", "connections", "local", "localharness_pb2.py")

    fd = load_descriptor(pb2)

    maps: dict = {}
    for msg in fd.message_type:
        collect_map_entries(msg, f"{fd.package}.{msg.name}", maps)

    out: list[str] = [
        "// GENERATED by scripts/gen_proto.py — do not edit by hand.",
        "//",
        f"// Source: {os.path.basename(root)} {os.path.join('google','antigravity','proto','localharness_pb2.py')}",
        "// Regenerate after every upstream release; see docs/upstream-parity.md.",
        "//",
        "// Rendered as proto3 with explicit presence on every singular field.",
        "// Upstream declares editions (edition=1001), which prost-build 0.12 cannot",
        "// parse, and this crate depends on presence: the step classifier and the",
        "// request dedup in src/local.rs are both built on `.is_some()`.",
        "",
        'syntax = "proto3";',
        "",
        f"package {fd.package};",
        "",
    ]

    for enum in fd.enum_type:
        render_enum(enum, "", out)
        out.append("")
    for msg in fd.message_type:
        render_message(msg, maps, "", "", out)
        out.append("")

    print("\n".join(out).rstrip() + "\n")


if __name__ == "__main__":
    main()
