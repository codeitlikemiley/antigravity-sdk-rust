#!/usr/bin/env python3
"""Probe a `localharness` binary's initialization handshake.

Speaks the harness protocol directly — length-prefixed binary `InputConfig` on
stdin, `OutputConfig` on stdout, then protojson over the WebSocket — so the
`InitializeConversationEvent` JSON can be varied without going through either
SDK. Used to establish empirically which `HarnessConfig` shapes a given harness
release accepts, and what it does when it rejects one.

Usage:
    pip install protobuf websockets
    # unpack any google-antigravity wheel; it ships bin/localharness and the
    # generated *_pb2.py the descriptors are read from
    python3 scripts/probe_harness.py /path/to/unpacked-wheel

Prints, for each probe case, either the harness's first frame or the error it
died with.
"""

from __future__ import annotations

import ast
import asyncio
import json
import os
import struct
import subprocess
import sys
import tempfile

from google.protobuf import descriptor_pb2, descriptor_pool, message_factory

import websockets

WS_TIMEOUT_SECONDS = 8


def _file_descriptor(pb2_path: str) -> descriptor_pb2.FileDescriptorProto:
    """Pulls the serialized FileDescriptorProto out of a generated _pb2.py."""
    tree = ast.parse(open(pb2_path, "rb").read().decode("utf-8", "replace"))
    for node in ast.walk(tree):
        if isinstance(node, ast.Call) and getattr(node.func, "attr", "") == "AddSerializedFile":
            fd = descriptor_pb2.FileDescriptorProto()
            fd.ParseFromString(ast.literal_eval(node.args[0]))
            return fd
    raise SystemExit(f"no AddSerializedFile call in {pb2_path}")


def _load_messages(root: str):
    pkg = os.path.join(root, "google", "antigravity")
    pb2 = os.path.join(pkg, "proto", "localharness_pb2.py")
    if not os.path.exists(pb2):  # pre-0.1.8 layout
        pb2 = os.path.join(pkg, "connections", "local", "localharness_pb2.py")

    pool = descriptor_pool.DescriptorPool()
    content = os.path.join(pkg, "proto", "content_pb2.py")
    if os.path.exists(content):  # 0.1.9+ localharness.proto imports it
        from google.protobuf import duration_pb2, struct_pb2

        for dep in (duration_pb2, struct_pb2):
            pool.Add(descriptor_pb2.FileDescriptorProto.FromString(dep.DESCRIPTOR.serialized_pb))
        pool.Add(_file_descriptor(content))
    pool.Add(_file_descriptor(pb2))

    def msg(name: str):
        return message_factory.GetMessageClass(pool.FindMessageTypeByName(name))

    return msg("antigravity.localharness.InputConfig"), msg("antigravity.localharness.OutputConfig")


async def probe(root: str, label: str, init_event: str, store: str) -> None:
    input_config_cls, output_config_cls = _load_messages(root)
    binary = os.path.join(root, "google", "antigravity", "bin", "localharness")

    proc = subprocess.Popen(
        [binary], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE
    )
    try:
        serialized = input_config_cls(storage_directory=store).SerializeToString()
        proc.stdin.write(struct.pack("<I", len(serialized)) + serialized)
        proc.stdin.flush()

        raw_len = proc.stdout.read(4)
        if not raw_len:
            print(f"[{label}] harness never wrote OutputConfig: {proc.stderr.read().decode()[:800]}")
            return
        output_config = output_config_cls()
        output_config.ParseFromString(proc.stdout.read(struct.unpack("<I", raw_len)[0]))

        async with websockets.connect(
            f"ws://127.0.0.1:{output_config.port}/",
            additional_headers={"x-goog-api-key": output_config.api_key},
            max_size=None,
        ) as ws:
            await ws.send(init_event)
            resp = await asyncio.wait_for(ws.recv(), timeout=WS_TIMEOUT_SECONDS)
            print(f"[{label}] ACCEPTED -> {str(resp)[:400]}")
    except asyncio.TimeoutError:
        print(f"[{label}] no frame within {WS_TIMEOUT_SECONDS}s (connection still open)")
    except Exception as exc:  # noqa: BLE001 - the failure mode is the result
        print(f"[{label}] REJECTED -> {type(exc).__name__}: {exc}")
    finally:
        proc.kill()
        stderr = proc.stderr.read().decode().strip()
        if stderr:
            print(f"[{label}] harness stderr: {stderr[-600:]}")


def main() -> None:
    if len(sys.argv) < 2:
        raise SystemExit(__doc__)
    root = os.path.abspath(sys.argv[1])
    store = tempfile.mkdtemp(prefix="harness-probe-")
    workspace = os.path.abspath(".")
    cascade = "probe-cascade-0123456789abcdef0123456789"

    def config(**fields) -> str:
        base = {"workspaces": [{"filesystemWorkspace": {"directory": workspace}}]}
        base.update(fields)
        return json.dumps({"config": base})

    models = [{"name": "gemini-3.6-flash", "types": ["MODEL_TYPE_TEXT"], "geminiApiEndpoint": {"apiKey": "PROBE"}}]

    cases = [
        # Pre-0.1.4 shape — what antigravity-sdk-rust sends today.
        ("0.1.1 geminiConfig", config(cascadeId=cascade, geminiConfig={"apiKey": "PROBE", "models": {"default": {"name": "gemini-3.5-flash"}}})),
        # 0.1.4+ shape, new conversation (harness allocates the id).
        ("0.1.9 models[], empty cascadeId", config(cascadeId="", models=models)),
        # 0.1.4+ shape, caller-chosen id, no continuation mode.
        ("0.1.9 models[], caller id, no sessionContinuationMode", config(cascadeId=cascade, models=models)),
        # 0.1.7+ continuation mode, caller-chosen id.
        ("0.1.9 models[], caller id, CREATE_OR_RESUME", config(cascadeId=cascade, models=models, sessionContinuationMode="CREATE_OR_RESUME")),
    ]

    for label, event in cases:
        asyncio.run(probe(root, label, event, store))
        print("-" * 72)


if __name__ == "__main__":
    main()
