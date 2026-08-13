#!/usr/bin/env python3
"""Capture and verify the immutable CAP-023 accepted-head evidence record.

The tool intentionally uses only the Python standard library.  Target artifact
byte production is closed over canonical Git blobs, pinned Rust/LLVM payloads,
and the embedded launch-support sources.  Runner substrate details are retained
as observations and are never promoted to immutable inputs.
"""

from __future__ import annotations

import argparse
import base64
import hashlib
import io
import json
import os
import pathlib
import platform as host_platform
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import urllib.request
from typing import Any, Iterable, Mapping, Sequence


AUTHORIZATION_HEAD = "afd251d1f653649c0bf0ad6d000c62698fce840a"
SUBJECT_COMMIT = "918c9222eb61e2435e18847e30b946cd08013238"
SUBJECT_TREE = "aba2876644b0183ab877b2e28d5e14001328c99a"
SUBJECT_PARENTS = [
    "e9b281504446465cfc8fcbe17c65cce92df0e83a",
    "d21c91fc312c70c47c6bb865ba1465e762255f0c",
]
COMPILER_TREE = "0ba0d06899b7e95d6b5b6f90a14804d18651806c"
CLAIM_ID = "aero_cap023_inference_correctness_918c9222_20260813"
CLAIM_STATUS = "verified_correctness_reproducibility_only"
SCHEMA_ID = "aero-cap023-inference-evidence-v1"
TOOL_ID = "cap024-inference-evidence-v1"
ORACLE_ID = "cap023-relu-argmax-inference-oracle-v1"
FRESH_OBSERVATIONS_ID = "cap024-fresh-observations-v1"
RUST_VERSION = "1.97.1"
RUST_COMMIT = "8bab26f4f68e0e26f0bb7960be334d5b520ea452"
LLVM_VERSION = "22.1.8"
LLVM_VERSION_BANNER = "LLVM version 22.1.8"
CLANG_VERSION_BANNER = "clang version 22.1.8"
LLD_VERSION_BANNER = "LLD 22.1.8"
TARGET_BYTE_ENVIRONMENT_ANCHOR = '"inheritance": "none"'
CRATES_IO_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"
CRATES_IO_DOWNLOAD_PREFIX = "https://static.crates.io/crates/"
LOCKFILE_VENDOR_BOUNDARY = "target-byte dependency vendor must be lockfile-complete"

REPOSITORY_ROOT = pathlib.Path(__file__).resolve().parents[1]
TOOL_PATH = "tools/cap024_inference_evidence.py"
WORKFLOW_PATH = ".github/workflows/cap023-evidence.yml"
SCHEMA_PATH = "claim-verification/schemas/aero-cap023-inference-evidence-v1.schema.json"
BUNDLE_DIRECTORY = (
    "claim-verification/results/"
    "aero_cap023_inference_correctness_918c9222_20260813"
)
REPRODUCE_PATH = f"{BUNDLE_DIRECTORY}/REPRODUCE.md"
ORACLE_PATH = f"{BUNDLE_DIRECTORY}/oracle.json"
MANIFEST_PATH = f"{BUNDLE_DIRECTORY}/manifest.json"
TRANSPORT_ENV = "CAP024_TRANSPORT_ROOT"
LINUX_JOB_RESULT_ENV = "CAP024_LINUX_JOB_RESULT"
WINDOWS_JOB_RESULT_ENV = "CAP024_WINDOWS_JOB_RESULT"
REQUIRED_BUNDLE_FILES = {"REPRODUCE.md", "manifest.json", "oracle.json"}
PLATFORM_NAMES = ["linux-x86_64", "windows-x86_64"]
ARTIFACT_NAMES = [
    "llvm",
    "bitcode",
    "assembly",
    "executable_o0",
    "executable_o2",
]
TOOL_NAMES = ["cargo", "rustc", "clang", "lld", "opt", "llvm-as", "llc"]
MANIFEST_FIELDS = {
    "authorization_head",
    "claim_id",
    "failures",
    "inputs",
    "limitations",
    "oracle",
    "platforms",
    "replay",
    "reproduce",
    "schema",
    "schema_id",
    "schema_version",
    "scope",
    "subject",
    "support",
    "tool",
    "transport",
    "workflow",
}
ALLOWED_PATHS = [
    "TASK_LEDGER.md",
    WORKFLOW_PATH,
    TOOL_PATH,
    SCHEMA_PATH,
    MANIFEST_PATH,
    ORACLE_PATH,
    REPRODUCE_PATH,
    "claim-verification/claims.json",
    "src/compiler/tests/cap024_claim_verification_contract_tests.rs",
    "src/compiler/tests/cli_status_contract_tests.rs",
]

FROZEN_INPUTS = [
    {
        "path": "examples/fixed_int_array_v0/relu_argmax_inference.aero",
        "blob": "5d5fe74e4acc351cb4326e85c4d69f320a37f3c6",
        "sha256": "8244ca26fc90ce708801e12ec6a7192bdedfd01e1a1429c1479d36e233b1bb6c",
        "size": 8224,
    },
    {
        "path": ".github/workflows/rust.yml",
        "blob": "888a1d6b699725ebdd8b8fd6c762c1b58cd823a3",
        "sha256": "32c820df765c6f42025d46a9f95049610fb8c301233f51920c7182fda74a92f5",
        "size": 264585,
    },
    {
        "path": "src/compiler/tests/fixed_int_array_profile_tests.rs",
        "blob": "959033d0fd255b947d16aa83efe914b517ced412",
        "sha256": "6300d3e2a9ef51c270c9ea876a54e70be3fae0e55ccaab5bb81a060a36af5103",
        "size": 257332,
    },
    {
        "path": "src/compiler/Cargo.toml",
        "blob": "156dee0fc73aad0bf832c216edbfc9d13fb70012",
        "sha256": "ee0ab0da24d5706101b37fdf94940fe863e097bcc02b0752b0bccaddf48ab96f",
        "size": 1072,
    },
    {
        "path": "src/compiler/Cargo.lock",
        "blob": "24c4729076801853f7bebb4a3269c050f31b3a5a",
        "sha256": "076d1d4f06ed35627c45a93428aab3705fceafcada5f09ae1597ada6922ff280",
        "size": 26063,
    },
]

LINUX_LLVM_ARCHIVE_NAME = "LLVM-22.1.8-Linux-X64.tar.xz"
LINUX_LLVM_ARCHIVE_SHA256 = (
    "df0e1ecf16caf3489a272a5eea4eec9b0d82878f6477fa309504f918a0006384"
)
LINUX_LLVM_ARCHIVE_SIZE = 1938859476
WINDOWS_LLVM_ARCHIVE_NAME = "clang+llvm-22.1.8-x86_64-pc-windows-msvc.tar.xz"
WINDOWS_LLVM_ARCHIVE_SHA256 = (
    "d96c2cc1736f4eb7fa43cb9bbdf56d93551a9ae0a9aadb9c99c3c3b2b712a234"
)
WINDOWS_LLVM_ARCHIVE_SIZE = 862053924

LINUX_START = (
    b'.text\n.p2align 4\n.globl _start\n.type _start,@function\n_start:\n'
    b'        callq main\n        movl %eax, %edi\n        movl $60, %eax\n'
    b'        syscall\n.size _start, .-_start\n'
    b'.section .note.GNU-stack,"",@progbits\n'
)
LINUX_START_SHA256 = "b95dbd79fd7b976862149e5635e148b9a9d2bbf20b2c3912a1f8d76c227379bb"
LINUX_START_SIZE = 205
WINDOWS_CHKSTK = (
    b".text\n.p2align 2\n.globl __chkstk\n__chkstk:\n"
    b"        pushq %rcx\n        pushq %rax\n        cmpq $0x1000, %rax\n"
    b"        leaq 24(%rsp), %rcx\n        jb 1f\n2:\n"
    b"        subq $0x1000, %rcx\n        testq %rcx, (%rcx)\n"
    b"        subq $0x1000, %rax\n        cmpq $0x1000, %rax\n"
    b"        ja 2b\n1:\n        subq %rax, %rcx\n"
    b"        testq %rcx, (%rcx)\n        popq %rax\n"
    b"        popq %rcx\n        retq\n"
)
WINDOWS_CHKSTK_SHA256 = (
    "b971f9c51534aff82d774c26b6a6f2312a3beeac5e1710a69f3d88bd5671f376"
)
WINDOWS_CHKSTK_SIZE = 378

CHECKOUT_ACTION = "actions/checkout@11d5960a326750d5838078e36cf38b85af677262"
UPLOAD_ACTION = "actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02"
DOWNLOAD_ACTION = "actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093"

COMMAND_NAMES = [
    "clean_before",
    "compiler_build_first",
    "compiler_build_second",
    "aero_build_llvm_first",
    "aero_build_llvm_second",
    "llvm_verify_first",
    "llvm_verify_second",
    "llvm_assemble_first",
    "llvm_assemble_second",
    "machine_verify_first",
    "machine_verify_second",
    "link_o0_first",
    "link_o0_second",
    "link_o2_first",
    "link_o2_second",
    "native_o0_first",
    "native_o0_second",
    "native_o2_first",
    "native_o2_second",
    "public_run",
    "clean_after",
]

REPLAY_EXCLUSIONS = [
    "/platforms/0/observations/runner_image",
    "/platforms/0/observations/kernel",
    "/platforms/0/compiler_executables/first/sha256",
    "/platforms/0/compiler_executables/first/size",
    "/platforms/0/compiler_executables/second/sha256",
    "/platforms/0/compiler_executables/second/size",
    "/platforms/0/commands/aero_build_llvm_first/stdout/base64",
    "/platforms/0/commands/aero_build_llvm_first/stdout/sha256",
    "/platforms/0/commands/aero_build_llvm_first/stdout/size",
    "/platforms/0/commands/aero_build_llvm_first/stderr/base64",
    "/platforms/0/commands/aero_build_llvm_first/stderr/sha256",
    "/platforms/0/commands/aero_build_llvm_first/stderr/size",
    "/platforms/0/commands/aero_build_llvm_second/stdout/base64",
    "/platforms/0/commands/aero_build_llvm_second/stdout/sha256",
    "/platforms/0/commands/aero_build_llvm_second/stdout/size",
    "/platforms/0/commands/aero_build_llvm_second/stderr/base64",
    "/platforms/0/commands/aero_build_llvm_second/stderr/sha256",
    "/platforms/0/commands/aero_build_llvm_second/stderr/size",
    "/platforms/0/commands/public_run/stdout/base64",
    "/platforms/0/commands/public_run/stdout/sha256",
    "/platforms/0/commands/public_run/stdout/size",
    "/platforms/0/commands/public_run/stderr/base64",
    "/platforms/0/commands/public_run/stderr/sha256",
    "/platforms/0/commands/public_run/stderr/size",
    "/platforms/1/observations/runner_image",
    "/platforms/1/observations/kernel",
    "/platforms/1/compiler_executables/first/sha256",
    "/platforms/1/compiler_executables/first/size",
    "/platforms/1/compiler_executables/second/sha256",
    "/platforms/1/compiler_executables/second/size",
    "/platforms/1/commands/aero_build_llvm_first/stdout/base64",
    "/platforms/1/commands/aero_build_llvm_first/stdout/sha256",
    "/platforms/1/commands/aero_build_llvm_first/stdout/size",
    "/platforms/1/commands/aero_build_llvm_first/stderr/base64",
    "/platforms/1/commands/aero_build_llvm_first/stderr/sha256",
    "/platforms/1/commands/aero_build_llvm_first/stderr/size",
    "/platforms/1/commands/aero_build_llvm_second/stdout/base64",
    "/platforms/1/commands/aero_build_llvm_second/stdout/sha256",
    "/platforms/1/commands/aero_build_llvm_second/stdout/size",
    "/platforms/1/commands/aero_build_llvm_second/stderr/base64",
    "/platforms/1/commands/aero_build_llvm_second/stderr/sha256",
    "/platforms/1/commands/aero_build_llvm_second/stderr/size",
    "/platforms/1/commands/public_run/stdout/base64",
    "/platforms/1/commands/public_run/stdout/sha256",
    "/platforms/1/commands/public_run/stdout/size",
    "/platforms/1/commands/public_run/stderr/base64",
    "/platforms/1/commands/public_run/stderr/sha256",
    "/platforms/1/commands/public_run/stderr/size",
]

EMPTY_SHA256 = hashlib.sha256(b"").hexdigest()
SELF_TEST_SUCCESS = '{"mode":"self-test","ok":true}'
VALIDATE_SUCCESS = '{"mode":"validate","ok":true}'
REPLAY_SUCCESS = '{"mode":"replay","ok":true}'
CAPTURE_SUCCESS = '{"mode":"capture","ok":true}'
AGGREGATE_SUCCESS = '{"mode":"aggregate","ok":true}'

ORDINARY = [2, 3, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]
WRAPPING = [
    2, 3, 2, 2, -3, 5, 2147483647, 4, -2, -2147483648, -1, 3,
    2147483647, 2147483647, 2, 7, -2147483648, -3, 13, -7,
]
ACTIVATION = [2, 3, 2, 1, 1, 1, -1, -1, -1, 1, 0, -1, 2, 0, 1, 2, 3, 4, 5, 4]
TIE = [2, 3, 2, 1, 2, 3, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 3, 0, 0, 0]
MALFORMED_FIRST = [1, 3, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]
MALFORMED_SECOND = [2, 4, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]
MALFORMED_THIRD = [2, 3, 1, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]
SOURCE_RECORDS = [
    ("ordinary", ORDINARY),
    ("wrapping", WRAPPING),
    ("activation_boundary", ACTIVATION),
    ("tie", TIE),
    ("malformed_header_0", MALFORMED_FIRST),
    ("malformed_header_1", MALFORMED_SECOND),
    ("malformed_header_2", MALFORMED_THIRD),
]

LIMITATIONS = [
    "Correctness and reproducibility evidence only; byte sizes are footprint facts.",
    "Target artifacts reproduce only inside their stated platform and pinned-tool boundary.",
    "Runner and kernel identities are observations, not immutable inputs.",
    "No timing, resource-use, ABI, safety, accelerator, or general-inference claim.",
]


class EvidenceError(Exception):
    """A fail-closed evidence contract violation."""


class ClosedArgumentParser(argparse.ArgumentParser):
    """Route invocation defects through the canonical failure interface."""

    def error(self, message: str) -> None:
        raise EvidenceError(f"invalid invocation: {message}")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise EvidenceError(message)


def sha256_bytes(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def canonical_json_bytes(value: Any) -> bytes:
    return (
        json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True)
        + "\n"
    ).encode("utf-8")


def _reject_float(value: str) -> None:
    raise EvidenceError(f"floating JSON number is forbidden: {value}")


def _reject_constant(value: str) -> None:
    raise EvidenceError(f"non-finite JSON number is forbidden: {value}")


def _closed_pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise EvidenceError(f"duplicate JSON key is forbidden: {key!r}")
        result[key] = value
    return result


def parse_json_bytes(payload: bytes, label: str, canonical: bool = False) -> Any:
    require(not payload.startswith(b"\xef\xbb\xbf"), f"{label} has a UTF-8 BOM")
    try:
        text = payload.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise EvidenceError(f"{label} is not UTF-8: {exc}") from exc
    try:
        value = json.loads(
            text,
            object_pairs_hook=_closed_pairs,
            parse_float=_reject_float,
            parse_constant=_reject_constant,
        )
    except (json.JSONDecodeError, TypeError, ValueError) as exc:
        raise EvidenceError(f"{label} is not strict JSON: {exc}") from exc
    if canonical:
        require(payload == canonical_json_bytes(value), f"{label} is not sorted compact canonical JSON plus exactly one LF")
    return value


def read_json(path: pathlib.Path, label: str, canonical: bool = True) -> Any:
    try:
        payload = path.read_bytes()
    except OSError as exc:
        raise EvidenceError(f"cannot read {label} at {path}: {exc}") from exc
    return parse_json_bytes(payload, label, canonical)


def write_json(path: pathlib.Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(canonical_json_bytes(value))


def exact_keys(value: Any, expected: Iterable[str], label: str) -> dict[str, Any]:
    require(isinstance(value, dict), f"{label} must be an object")
    expected_set = set(expected)
    require(set(value) == expected_set, f"{label} keys differ from {sorted(expected_set)}")
    return value


def valid_sha256(value: Any) -> bool:
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value) is not None


def valid_git_hash(value: Any) -> bool:
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{40}", value) is not None


def byte_record(payload: bytes) -> dict[str, Any]:
    return {
        "base64": base64.b64encode(payload).decode("ascii"),
        "sha256": sha256_bytes(payload),
        "size": len(payload),
    }


def decode_byte_record(value: Any, label: str) -> bytes:
    record = exact_keys(value, {"base64", "sha256", "size"}, label)
    require(valid_sha256(record["sha256"]), f"{label}.sha256 is malformed")
    require(isinstance(record["size"], int) and record["size"] >= 0, f"{label}.size is invalid")
    require(isinstance(record["base64"], str), f"{label}.base64 must be text")
    try:
        payload = base64.b64decode(record["base64"], validate=True)
    except (ValueError, TypeError) as exc:
        raise EvidenceError(f"{label}.base64 is invalid: {exc}") from exc
    require(base64.b64encode(payload).decode("ascii") == record["base64"], f"{label}.base64 is not canonical")
    require(len(payload) == record["size"], f"{label}.size does not match decoded bytes")
    require(sha256_bytes(payload) == record["sha256"], f"{label}.sha256 does not match decoded bytes")
    return payload


def file_identity(path: pathlib.Path) -> tuple[str, int]:
    payload = path.read_bytes()
    return sha256_bytes(payload), len(payload)


def wrap_i32(value: int) -> int:
    value &= 0xFFFFFFFF
    return value - 0x100000000 if value >= 0x80000000 else value


def wrapping_add(left: int, right: int) -> int:
    return wrap_i32(left + right)


def wrapping_mul(left: int, right: int) -> int:
    return wrap_i32(left * right)


def inference_oracle(record: Sequence[int]) -> dict[str, Any]:
    require(len(record) == 20 and all(isinstance(v, int) for v in record), "oracle record must contain exactly twenty integer lanes")
    if list(record[:3]) != [2, 3, 2]:
        return {
            "first_products": [0] * 6,
            "raw": [0] * 2,
            "biased_hidden": [0] * 2,
            "hidden": [0] * 2,
            "second_products": [0] * 4,
            "raw_logits": [0] * 2,
            "logits": [0] * 2,
            "result": [0] * 8,
        }
    inputs = list(record[3:6])
    first_weights = list(record[6:12])
    first_bias = list(record[12:14])
    second_weights = list(record[14:18])
    second_bias = list(record[18:20])
    first_products = [
        wrapping_mul(first_weights[0], inputs[0]),
        wrapping_mul(first_weights[1], inputs[1]),
        wrapping_mul(first_weights[2], inputs[2]),
        wrapping_mul(first_weights[3], inputs[0]),
        wrapping_mul(first_weights[4], inputs[1]),
        wrapping_mul(first_weights[5], inputs[2]),
    ]
    raw = [
        wrapping_add(wrapping_add(first_products[0], first_products[1]), first_products[2]),
        wrapping_add(wrapping_add(first_products[3], first_products[4]), first_products[5]),
    ]
    biased_hidden = [wrapping_add(raw[i], first_bias[i]) for i in range(2)]
    hidden = [value if value > 0 else 0 for value in biased_hidden]
    second_products = [
        wrapping_mul(second_weights[0], hidden[0]),
        wrapping_mul(second_weights[1], hidden[1]),
        wrapping_mul(second_weights[2], hidden[0]),
        wrapping_mul(second_weights[3], hidden[1]),
    ]
    raw_logits = [
        wrapping_add(second_products[0], second_products[1]),
        wrapping_add(second_products[2], second_products[3]),
    ]
    logits = [wrapping_add(raw_logits[i], second_bias[i]) for i in range(2)]
    chosen = 1 if logits[1] > logits[0] else 0
    return {
        "first_products": first_products,
        "raw": raw,
        "biased_hidden": biased_hidden,
        "hidden": hidden,
        "second_products": second_products,
        "raw_logits": raw_logits,
        "logits": logits,
        "result": [1, raw[0], raw[1], hidden[0], hidden[1], logits[0], logits[1], chosen],
    }


def expected_oracle() -> dict[str, Any]:
    records = []
    for name, source in SOURCE_RECORDS:
        computed = inference_oracle(source)
        records.append(
            {
                "biased_hidden": computed["biased_hidden"],
                "first_products": computed["first_products"],
                "header_valid": source[:3] == [2, 3, 2],
                "hidden": computed["hidden"],
                "lane_count": 20,
                "logits": computed["logits"],
                "name": name,
                "raw": computed["raw"],
                "raw_logits": computed["raw_logits"],
                "result": computed["result"],
                "second_products": computed["second_products"],
                "source": list(source),
                "source_after_call": list(source),
                "source_preserved": True,
            }
        )
    return {
        "arithmetic": "signed-i32-two-complement-wrapping",
        "header": [2, 3, 2],
        "oracle_id": ORACLE_ID,
        "records": records,
        "rules": {
            "argmax": "signed-strict-greater-lower-index-tie",
            "header_gate": "exact-[2,3,2]-else-eight-zeros",
            "layout": "row-major-matvec-2x3-then-2x2",
            "logits": "two-wrapping-i32-biased-logits",
            "relu": "strict-positive-else-zero",
            "wrapping": "signed-i32-two-complement-every-mul-add",
        },
        "sentinel": 91,
        "source": dict(FROZEN_INPUTS[0]),
        "source_preservation_lanes": 140,
        "version": 1,
    }


def validate_oracle(value: Any) -> None:
    require(value == expected_oracle(), "oracle differs from the explicit signed wrapping arithmetic reference")


def validate_closed_schema_nodes(value: Any, path: str = "schema") -> None:
    if isinstance(value, list):
        for index, child in enumerate(value):
            validate_closed_schema_nodes(child, f"{path}/{index}")
        return
    if not isinstance(value, dict):
        return
    if value.get("type") == "object":
        require(value.get("additionalProperties") is False, f"{path} is not a closed object schema")
        properties = value.get("properties")
        required = value.get("required")
        require(isinstance(properties, dict), f"{path}.properties is missing")
        require(isinstance(required, list), f"{path}.required is missing")
        require(set(required) == set(properties), f"{path} does not require every exact property")
    if value.get("type") == "array":
        require(all(key in value for key in ("items", "minItems", "maxItems", "uniqueItems")), f"{path} array closure is incomplete")
    for key, child in value.items():
        validate_closed_schema_nodes(child, f"{path}/{key}")


def _json_type_matches(value: Any, expected: str) -> bool:
    if expected == "object":
        return isinstance(value, dict)
    if expected == "array":
        return isinstance(value, list)
    if expected == "string":
        return isinstance(value, str)
    if expected == "integer":
        return isinstance(value, int) and not isinstance(value, bool)
    if expected == "number":
        return isinstance(value, (int, float)) and not isinstance(value, bool)
    if expected == "boolean":
        return isinstance(value, bool)
    if expected == "null":
        return value is None
    return False


def validate_against_schema(value: Any, schema: Any, path: str = "$") -> None:
    require(isinstance(schema, dict), f"schema node {path} must be an object")
    if "const" in schema:
        require(value == schema["const"], f"{path} differs from schema const")
    if "enum" in schema:
        require(value in schema["enum"], f"{path} is outside schema enum")
    if "oneOf" in schema:
        matches = 0
        for option in schema["oneOf"]:
            try:
                validate_against_schema(value, option, path)
            except EvidenceError:
                continue
            matches += 1
        require(matches == 1, f"{path} must match exactly one schema branch")
        return
    expected_type = schema.get("type")
    if expected_type is not None:
        require(isinstance(expected_type, str) and _json_type_matches(value, expected_type), f"{path} has the wrong JSON type")
    if isinstance(value, dict) and expected_type == "object":
        properties = schema.get("properties", {})
        required = schema.get("required", [])
        for key in required:
            require(key in value, f"{path} omitted required field {key}")
        if schema.get("additionalProperties") is False:
            require(set(value) <= set(properties), f"{path} contains an unknown field")
        for key, child in value.items():
            if key in properties:
                validate_against_schema(child, properties[key], f"{path}/{key}")
    if isinstance(value, list) and expected_type == "array":
        if "minItems" in schema:
            require(len(value) >= schema["minItems"], f"{path} is shorter than minItems")
        if "maxItems" in schema:
            require(len(value) <= schema["maxItems"], f"{path} is longer than maxItems")
        if schema.get("uniqueItems"):
            canonical = [canonical_json_bytes(item) for item in value]
            require(len(canonical) == len(set(canonical)), f"{path} contains duplicate array values")
        if "items" in schema:
            for index, item in enumerate(value):
                validate_against_schema(item, schema["items"], f"{path}/{index}")
    if isinstance(value, str) and "pattern" in schema:
        require(re.fullmatch(schema["pattern"], value) is not None, f"{path} does not match schema pattern")
    if isinstance(value, int) and not isinstance(value, bool) and "minimum" in schema:
        require(value >= schema["minimum"], f"{path} is below schema minimum")


def validate_schema(value: Any) -> None:
    schema = exact_keys(
        value,
        {"$defs", "$id", "$schema", "additionalProperties", "properties", "required", "title", "type"},
        "schema",
    )
    require(schema["$schema"] == "https://json-schema.org/draft/2020-12/schema", "schema dialect drifted")
    require(schema["$id"] == SCHEMA_ID, "schema ID drifted")
    require(schema["type"] == "object" and schema["additionalProperties"] is False, "schema root is not closed")
    require(set(schema["required"]) == MANIFEST_FIELDS, "schema required fields drifted")
    require(set(schema["properties"]) == MANIFEST_FIELDS, "schema properties drifted")
    require(set(schema["$defs"]) == {"artifact_pair", "byte_record", "command"}, "schema definitions drifted")
    validate_closed_schema_nodes(schema)


def executable_suffix(platform_name: str) -> str:
    return ".exe" if platform_name == "windows-x86_64" else ""


def artifact_path(platform_name: str, production: str, artifact: str) -> str:
    if artifact == "llvm":
        return f"${{WORK}}/{platform_name}/{production}/inference.ll"
    if artifact == "bitcode":
        return f"${{WORK}}/{platform_name}/{production}/inference.bc"
    if artifact == "assembly":
        return f"${{WORK}}/{platform_name}/{production}/inference.s"
    suffix = executable_suffix(platform_name)
    if artifact == "executable_o0":
        return f"${{WORK}}/{platform_name}/{production}/inference-o0{suffix}"
    if artifact == "executable_o2":
        return f"${{WORK}}/{platform_name}/{production}/inference-o2{suffix}"
    raise EvidenceError(f"unknown artifact {artifact}")


def compiler_path(platform_name: str, production: str) -> str:
    return f"${{WORK}}/{platform_name}/cargo-{production}/release/aero{executable_suffix(platform_name)}"


def artifact_producer(artifact: str, production: str) -> str:
    prefixes = {
        "llvm": "aero_build_llvm",
        "bitcode": "llvm_assemble",
        "assembly": "machine_verify",
        "executable_o0": "link_o0",
        "executable_o2": "link_o2",
    }
    require(artifact in prefixes, f"unknown artifact producer for {artifact}")
    return f"{prefixes[artifact]}_{production}"


def expected_tool_path(platform_name: str, tool: str) -> str:
    suffix = executable_suffix(platform_name)
    if tool in {"cargo", "rustc"}:
        return f"${{RUST}}/bin/{tool}{suffix}"
    if tool == "lld":
        return "${LLVM}/bin/lld-link.exe" if platform_name == "windows-x86_64" else "${LLVM}/bin/ld.lld"
    return f"${{LLVM}}/bin/{tool}{suffix}"


def command_inheritance(name: str) -> str:
    if name in {
        "compiler_build_first",
        "compiler_build_second",
        "aero_build_llvm_first",
        "aero_build_llvm_second",
        "public_run",
    }:
        return "runner-substrate-observation-only"
    return "none"


def command_environment(platform_name: str, name: str) -> dict[str, Any]:
    suffix = executable_suffix(platform_name)
    return {
        "inheritance": command_inheritance(name),
        "overrides": {
            "CARGO_HOME": f"${{WORK}}/{platform_name}/cargo-home",
            "CARGO_NET_OFFLINE": "true",
            "LC_ALL": "C",
            "RUSTC": f"${{RUST}}/bin/rustc{suffix}",
            "RUSTFLAGS": "-Awarnings",
            "TEMP": "${WORK}/tmp",
            "TMP": "${WORK}/tmp",
            "TZ": "UTC",
        },
        "path_prefix": ["${RUST}/bin", "${LLVM}/bin"],
        "selectors": {
            "cargo": f"${{RUST}}/bin/cargo{suffix}",
            "clang": f"${{LLVM}}/bin/clang{suffix}",
            "llc": f"${{LLVM}}/bin/llc{suffix}",
            "lld": expected_tool_path(platform_name, "lld"),
            "llvm_as": f"${{LLVM}}/bin/llvm-as{suffix}",
            "opt": f"${{LLVM}}/bin/opt{suffix}",
            "rustc": f"${{RUST}}/bin/rustc{suffix}",
        },
    }


def expected_command_spec(platform_name: str, name: str) -> dict[str, Any]:
    suffix = executable_suffix(platform_name)
    source = "${SUBJECT}/examples/fixed_int_array_v0/relu_argmax_inference.aero"
    support = (
        "${WORK}/windows-x86_64/windows-chkstk.S"
        if platform_name == "windows-x86_64"
        else "${WORK}/linux-x86_64/linux-start.S"
    )
    cargo = f"${{RUST}}/bin/cargo{suffix}"
    clang = f"${{LLVM}}/bin/clang{suffix}"
    opt = f"${{LLVM}}/bin/opt{suffix}"
    llvm_as = f"${{LLVM}}/bin/llvm-as{suffix}"
    llc = f"${{LLVM}}/bin/llc{suffix}"
    argv: list[str]
    consumes: list[str]
    produces: list[str]
    exit_code = 0
    if name in {"clean_before", "clean_after"}:
        phase = "clean-before" if name == "clean_before" else "clean-after"
        argv, consumes, produces = ["internal", phase, platform_name], [], []
    elif name.startswith("compiler_build_"):
        production = name.removeprefix("compiler_build_")
        output = compiler_path(platform_name, production)
        argv = [
            cargo,
            "build",
            "--quiet",
            "--locked",
            "--offline",
            "--release",
            "--bin",
            "aero",
            "--manifest-path",
            "${SUBJECT}/src/compiler/Cargo.toml",
            "--target-dir",
            f"${{WORK}}/{platform_name}/cargo-{production}",
        ]
        consumes = [
            "${SUBJECT}/src/compiler",
            "${SUBJECT}/src/compiler/Cargo.toml",
            "${SUBJECT}/src/compiler/Cargo.lock",
            f"${{WORK}}/{platform_name}/cargo-home/config.toml",
            f"${{WORK}}/{platform_name}/cargo-vendor",
        ]
        produces = [output]
    elif name.startswith("aero_build_llvm_"):
        production = name.removeprefix("aero_build_llvm_")
        compiler = compiler_path(platform_name, production)
        llvm = artifact_path(platform_name, production, "llvm")
        argv = [compiler, "build", source, "-o", llvm, "--require-llvm-verifier", "--language-profile", "exact-i32-array-v0"]
        consumes, produces = [compiler, source], [llvm]
    elif name.startswith("llvm_verify_"):
        production = name.removeprefix("llvm_verify_")
        llvm = artifact_path(platform_name, production, "llvm")
        argv, consumes, produces = [opt, "-passes=verify", "-disable-output", llvm], [llvm], []
    elif name.startswith("llvm_assemble_"):
        production = name.removeprefix("llvm_assemble_")
        llvm = artifact_path(platform_name, production, "llvm")
        bitcode = artifact_path(platform_name, production, "bitcode")
        argv, consumes, produces = [llvm_as, llvm, "-o", bitcode], [llvm], [bitcode]
    elif name.startswith("machine_verify_"):
        production = name.removeprefix("machine_verify_")
        llvm = artifact_path(platform_name, production, "llvm")
        assembly = artifact_path(platform_name, production, "assembly")
        argv = [llc, "-verify-machineinstrs", "-filetype=asm", llvm, "-o", assembly]
        consumes, produces = [llvm], [assembly]
    elif name.startswith(("link_o0_", "link_o2_")):
        if name.startswith("link_o0_"):
            optimization, production, artifact = "-O0", name.removeprefix("link_o0_"), "executable_o0"
        else:
            optimization, production, artifact = "-O2", name.removeprefix("link_o2_"), "executable_o2"
        llvm = artifact_path(platform_name, production, "llvm")
        executable = artifact_path(platform_name, production, artifact)
        argv = [clang, optimization, llvm, support, "-o", executable, "-nostdlib"]
        if platform_name == "windows-x86_64":
            argv += ["--ld-path=${LLVM}/bin/lld-link.exe", "-Wl,/entry:main,/subsystem:console,/nodefaultlib,/brepro"]
        else:
            argv += ["--ld-path=${LLVM}/bin/ld.lld", "-Wl,-e,_start,--build-id=none"]
        consumes, produces = [llvm, support], [executable]
    elif name.startswith(("native_o0_", "native_o2_")):
        parts = name.split("_")
        artifact = f"executable_{parts[1]}"
        executable = artifact_path(platform_name, parts[2], artifact)
        argv, consumes, produces, exit_code = [executable], [executable], [], 91
    elif name == "public_run":
        compiler = compiler_path(platform_name, "first")
        argv = [compiler, "run", source, "--language-profile", "exact-i32-array-v0"]
        consumes, produces, exit_code = [compiler, source], [], 91
    else:
        raise EvidenceError(f"unknown pipeline command {name}")
    return {
        "argv": argv,
        "consumes": consumes,
        "cwd": "${SUBJECT}",
        "env": command_environment(platform_name, name),
        "exit_code": exit_code,
        "name": name,
        "produces": produces,
    }


PIPELINE_COMMANDS = set(COMMAND_NAMES)
TOOL_VERSION_PROBES = {f"version:{name}" for name in TOOL_NAMES}
GIT_VERIFICATION_COMMANDS = {
    "git:autocrlf",
    "git:compiler-tree",
    "git:ls-tree",
    "git:object-format",
    "git:parents",
    "git:status",
    "git:tree",
}
subprocess_allowlist = PIPELINE_COMMANDS | TOOL_VERSION_PROBES | GIT_VERIFICATION_COMMANDS


def expected_parsed_version(tool: str) -> dict[str, str]:
    if tool == "rustc":
        return {"banner_kind": "rustc-vv", "commit": RUST_COMMIT, "version": RUST_VERSION}
    if tool == "cargo":
        return {"banner_kind": "cargo-vv", "version": RUST_VERSION}
    if tool == "clang":
        return {"banner_kind": "clang", "version": LLVM_VERSION}
    if tool == "lld":
        return {"banner_kind": "lld", "version": LLVM_VERSION}
    if tool in {"opt", "llvm-as", "llc"}:
        return {"banner_kind": "llvm", "version": LLVM_VERSION}
    raise EvidenceError(f"unknown tool version parser {tool}")


def parse_tool_banner(tool: str, stdout: bytes, stderr: bytes) -> dict[str, str]:
    require(stderr == b"", f"{tool} version probe wrote stderr")
    try:
        lines = stdout.decode("utf-8").splitlines()
    except UnicodeDecodeError as exc:
        raise EvidenceError(f"{tool} version stdout is not UTF-8: {exc}") from exc
    if tool == "rustc":
        require(any(line.startswith(f"rustc {RUST_VERSION} (") for line in lines), "rustc banner version drifted")
        require(f"release: {RUST_VERSION}" in lines, "rustc release line drifted")
        require(f"commit-hash: {RUST_COMMIT}" in lines, "rustc commit line drifted")
    elif tool == "cargo":
        require(any(line.startswith(f"cargo {RUST_VERSION} (") for line in lines), "cargo banner version drifted")
        require(f"release: {RUST_VERSION}" in lines, "cargo release line drifted")
        commit = next((line.removeprefix("commit-hash: ") for line in lines if line.startswith("commit-hash: ")), "")
        require(valid_git_hash(commit), "cargo raw commit identity is malformed")
    elif tool == "clang":
        require(any(line.startswith(CLANG_VERSION_BANNER) for line in lines), "clang banner drifted")
    elif tool == "lld":
        require(any(line == LLD_VERSION_BANNER or line.startswith(f"{LLD_VERSION_BANNER} (") for line in lines), "LLD banner drifted")
    else:
        require(any(line.strip() == LLVM_VERSION_BANNER for line in lines), f"{tool} LLVM banner drifted")
    return expected_parsed_version(tool)


def validate_tool_record(value: Any, platform_name: str, tool: str) -> None:
    record = exact_keys(value, {"path", "payload_sha256", "payload_size", "version"}, f"{platform_name}.{tool}")
    require(record["path"] == expected_tool_path(platform_name, tool), f"{platform_name}.{tool} path drifted")
    require(valid_sha256(record["payload_sha256"]), f"{platform_name}.{tool} payload hash is malformed")
    require(isinstance(record["payload_size"], int) and record["payload_size"] > 0, f"{platform_name}.{tool} payload size is invalid")
    version = exact_keys(record["version"], {"argv", "exit_code", "parsed", "stderr", "stdout"}, f"{platform_name}.{tool}.version")
    expected_argv = [record["path"], "-Vv" if tool in {"cargo", "rustc"} else "--version"]
    require(version["argv"] == expected_argv and version["exit_code"] == 0, f"{platform_name}.{tool} version command drifted")
    stdout = decode_byte_record(version["stdout"], f"{platform_name}.{tool}.version.stdout")
    stderr = decode_byte_record(version["stderr"], f"{platform_name}.{tool}.version.stderr")
    require(version["parsed"] == parse_tool_banner(tool, stdout, stderr), f"{platform_name}.{tool} parsed version is not derived from raw bytes")


def validate_command(value: Any, platform_name: str, name: str) -> None:
    command = exact_keys(
        value,
        {"argv", "consumes", "cwd", "env", "exit_code", "name", "produces", "stderr", "stdout"},
        f"{platform_name}.{name}",
    )
    expected = expected_command_spec(platform_name, name)
    for key in ("argv", "consumes", "cwd", "env", "exit_code", "name", "produces"):
        require(command[key] == expected[key], f"{platform_name}.{name}.{key} drifted")
    stdout = decode_byte_record(command["stdout"], f"{platform_name}.{name}.stdout")
    stderr = decode_byte_record(command["stderr"], f"{platform_name}.{name}.stderr")
    if name.startswith(("native_o0_", "native_o2_")):
        require(command["exit_code"] == 91 and stdout == b"" and stderr == b"", f"{platform_name}.{name} must exit 91 with empty streams")
    if name in {"aero_build_llvm_first", "aero_build_llvm_second", "public_run"}:
        require(stdout or stderr, f"{platform_name}.{name} omitted its raw diagnostic streams")
    if command_inheritance(name) == "none":
        require(command["env"]["inheritance"] == "none", "target-byte commands reject ambient inheritance")


def derive_public_semantics(commands: Mapping[str, Any], platform_name: str) -> dict[str, Any]:
    public = commands["public_run"]
    stdout = decode_byte_record(public["stdout"], f"{platform_name}.public_run.stdout")
    stderr = decode_byte_record(public["stderr"], f"{platform_name}.public_run.stderr")
    try:
        stdout_lines = stdout.decode("utf-8").splitlines()
        stderr_lines = stderr.decode("utf-8").splitlines()
    except UnicodeDecodeError as exc:
        raise EvidenceError(f"{platform_name} public wrapper streams are not UTF-8: {exc}") from exc
    require(stdout_lines.count("Exit code: 91") == 1, f"{platform_name} public stdout must contain exactly one whole Exit code: 91 line")
    require(not any(line.startswith("Exit code:") and line != "Exit code: 91" for line in stdout_lines), f"{platform_name} public stdout contains a conflicting exit line")
    require(not any(line.startswith("Output:") or line.startswith("Error output:") for line in stdout_lines), f"{platform_name} public stdout contains application output")
    require(not any(line.startswith("Exit code:") or line.startswith("Output:") or line.startswith("Error output:") for line in stderr_lines), f"{platform_name} public stderr contains wrapper status")
    empty = byte_record(b"")
    return {
        "application_stderr": empty,
        "application_stdout": empty,
        "exit_report_count": 1,
        "reported_exit_code": 91,
    }


def _validate_artifact_record(value: Any, platform_name: str, artifact: str, production: str) -> None:
    record = exact_keys(value, {"path", "producer_command", "sha256", "size"}, f"{platform_name}.{artifact}.{production}")
    require(record["path"] == artifact_path(platform_name, production, artifact), f"{platform_name}.{artifact}.{production} path drifted")
    require(record["producer_command"] == artifact_producer(artifact, production), f"{platform_name}.{artifact}.{production} producer drifted")
    require(valid_sha256(record["sha256"]), f"{platform_name}.{artifact}.{production} hash is malformed")
    require(isinstance(record["size"], int) and record["size"] > 0, f"{platform_name}.{artifact}.{production} size is invalid")


def validate_platform_record(value: Any, expected_name: str) -> None:
    platform_record = exact_keys(
        value,
        {"artifacts", "commands", "compiler_executables", "failures", "name", "observations", "public_semantics", "toolchain"},
        expected_name,
    )
    require(platform_record["name"] == expected_name, f"platform order/name drifted from {expected_name}")
    require(platform_record["failures"] == [], f"{expected_name} failure record is not empty")
    observations = exact_keys(platform_record["observations"], {"kernel", "runner_image"}, f"{expected_name}.observations")
    require(all(isinstance(observations[key], str) and observations[key] for key in observations), f"{expected_name} observations are incomplete")

    compiler_records = exact_keys(platform_record["compiler_executables"], {"first", "second"}, f"{expected_name}.compiler_executables")
    for production in ("first", "second"):
        record = exact_keys(compiler_records[production], {"path", "producer_command", "sha256", "size"}, f"{expected_name}.compiler.{production}")
        require(record["path"] == compiler_path(expected_name, production), f"{expected_name} compiler path drifted")
        require(record["producer_command"] == f"compiler_build_{production}", f"{expected_name} compiler producer drifted")
        require(valid_sha256(record["sha256"]) and isinstance(record["size"], int) and record["size"] > 0, f"{expected_name} compiler identity is incomplete")

    artifacts = exact_keys(platform_record["artifacts"], ARTIFACT_NAMES, f"{expected_name}.artifacts")
    for artifact in ARTIFACT_NAMES:
        pair = exact_keys(artifacts[artifact], {"first", "pair_equal", "second"}, f"{expected_name}.{artifact}")
        require(pair["pair_equal"] is True, f"{expected_name}.{artifact} pair equality is not asserted")
        for production in ("first", "second"):
            _validate_artifact_record(pair[production], expected_name, artifact, production)
        require(pair["first"]["sha256"] == pair["second"]["sha256"] and pair["first"]["size"] == pair["second"]["size"], f"{expected_name}.{artifact} productions differ")

    commands = exact_keys(platform_record["commands"], COMMAND_NAMES, f"{expected_name}.commands")
    for command_name in COMMAND_NAMES:
        validate_command(commands[command_name], expected_name, command_name)
    available = {
        "${SUBJECT}/src/compiler/Cargo.toml",
        "${SUBJECT}/src/compiler/Cargo.lock",
        "${SUBJECT}/src/compiler",
        "${SUBJECT}/examples/fixed_int_array_v0/relu_argmax_inference.aero",
        f"${{WORK}}/{expected_name}/cargo-home/config.toml",
        f"${{WORK}}/{expected_name}/cargo-vendor",
        "${WORK}/windows-x86_64/windows-chkstk.S" if expected_name == "windows-x86_64" else "${WORK}/linux-x86_64/linux-start.S",
    }
    for command_name in COMMAND_NAMES:
        command = commands[command_name]
        require(all(path in available for path in command["consumes"]), f"{expected_name}.{command_name} consumes an unavailable path")
        for path in command["produces"]:
            require(path not in available, f"{expected_name}.{command_name} duplicates a produced path")
            available.add(path)
    for production in ("first", "second"):
        compiler_record = compiler_records[production]
        require(compiler_record["path"] in commands[compiler_record["producer_command"]]["produces"], f"{expected_name} compiler producer/path linkage drifted")
        for artifact in ARTIFACT_NAMES:
            record = artifacts[artifact][production]
            require(record["path"] in commands[record["producer_command"]]["produces"], f"{expected_name}.{artifact}.{production} producer/path linkage drifted")

    toolchain = exact_keys(platform_record["toolchain"], {"archive_name", "archive_sha256", "archive_size", "llvm_version", "rust_commit", "rust_version", "setup_boundary", "tools"}, f"{expected_name}.toolchain")
    archive = platform_archive(expected_name)
    require(toolchain["archive_name"] == archive["name"], f"{expected_name} archive name drifted")
    require(toolchain["archive_sha256"] == archive["sha256"], f"{expected_name} archive hash drifted")
    require(toolchain["archive_size"] == archive["size"], f"{expected_name} archive size drifted")
    require(toolchain["llvm_version"] == LLVM_VERSION and toolchain["rust_version"] == RUST_VERSION and toolchain["rust_commit"] == RUST_COMMIT, f"{expected_name} toolchain pin drifted")
    require(toolchain["setup_boundary"] == "workflow-acquisition-only; every final tool payload and version is verified before capture", f"{expected_name} setup boundary drifted")
    tools = exact_keys(toolchain["tools"], TOOL_NAMES, f"{expected_name}.tools")
    for tool in TOOL_NAMES:
        validate_tool_record(tools[tool], expected_name, tool)
    tool_links = {
        "compiler_build_first": "cargo",
        "compiler_build_second": "cargo",
        "llvm_verify_first": "opt",
        "llvm_verify_second": "opt",
        "llvm_assemble_first": "llvm-as",
        "llvm_assemble_second": "llvm-as",
        "machine_verify_first": "llc",
        "machine_verify_second": "llc",
        "link_o0_first": "clang",
        "link_o0_second": "clang",
        "link_o2_first": "clang",
        "link_o2_second": "clang",
    }
    for command_name, tool in tool_links.items():
        require(commands[command_name]["argv"][0] == tools[tool]["path"], f"{expected_name}.{command_name} is not linked to recorded {tool}")
    lld_selector = f"--ld-path={tools['lld']['path']}"
    for command_name in ("link_o0_first", "link_o0_second", "link_o2_first", "link_o2_second"):
        require(lld_selector in commands[command_name]["argv"], f"{expected_name}.{command_name} is not linked to recorded lld")
    require(platform_record["public_semantics"] == derive_public_semantics(commands, expected_name), f"{expected_name} parsed public semantics drifted")


def platform_archive(platform_name: str) -> dict[str, Any]:
    if platform_name == "linux-x86_64":
        return {"name": LINUX_LLVM_ARCHIVE_NAME, "sha256": LINUX_LLVM_ARCHIVE_SHA256, "size": LINUX_LLVM_ARCHIVE_SIZE}
    if platform_name == "windows-x86_64":
        return {"name": WINDOWS_LLVM_ARCHIVE_NAME, "sha256": WINDOWS_LLVM_ARCHIVE_SHA256, "size": WINDOWS_LLVM_ARCHIVE_SIZE}
    raise EvidenceError(f"unsupported platform {platform_name}")


def validate_manifest(value: Any, schema: Any | None = None) -> None:
    manifest = exact_keys(value, MANIFEST_FIELDS, "manifest")
    require(manifest["authorization_head"] == AUTHORIZATION_HEAD, "manifest authorization head drifted")
    require(manifest["claim_id"] == CLAIM_ID, "manifest claim ID drifted")
    require(manifest["schema_id"] == SCHEMA_ID and manifest["schema_version"] == 1, "manifest schema identity drifted")
    require(manifest["scope"] == ALLOWED_PATHS, "manifest allowed path scope drifted")
    require(manifest["failures"] == [], "manifest failures must be explicitly empty")
    require(manifest["limitations"] == LIMITATIONS, "manifest limitations drifted")
    require(manifest["transport"] == "temporary-actions-text-only", "manifest transport drifted")
    subject = exact_keys(manifest["subject"], {"clean_after", "clean_before", "commit", "compiler_tree", "parents", "tree"}, "manifest.subject")
    require(subject == {"clean_after": True, "clean_before": True, "commit": SUBJECT_COMMIT, "compiler_tree": COMPILER_TREE, "parents": SUBJECT_PARENTS, "tree": SUBJECT_TREE}, "manifest subject identity drifted")
    require(manifest["inputs"] == FROZEN_INPUTS, "manifest canonical input identities drifted")
    for key, expected_path in {
        "schema": SCHEMA_PATH,
        "workflow": WORKFLOW_PATH,
        "tool": TOOL_PATH,
        "oracle": ORACLE_PATH,
        "reproduce": REPRODUCE_PATH,
    }.items():
        keys = {"id", "path", "sha256"} if key == "tool" else {"path", "sha256"}
        record = exact_keys(manifest[key], keys, f"manifest.{key}")
        require(record["path"] == expected_path and valid_sha256(record["sha256"]), f"manifest.{key} identity drifted")
        if key == "tool":
            require(record["id"] == TOOL_ID, "manifest tool ID drifted")
    support = exact_keys(manifest["support"], {"linux", "windows"}, "manifest.support")
    require(support == {
        "linux": {"path": "linux-start.S", "sha256": LINUX_START_SHA256, "size": LINUX_START_SIZE},
        "windows": {"path": "windows-chkstk.S", "sha256": WINDOWS_CHKSTK_SHA256, "size": WINDOWS_CHKSTK_SIZE},
    }, "manifest launch support identities drifted")
    replay = exact_keys(manifest["replay"], {"canonical_projection", "excluded_paths", "fresh_observations"}, "manifest.replay")
    require(replay["canonical_projection"] == "sorted-compact-json-plus-lf-v1", "manifest replay projection kind drifted")
    require(replay["excluded_paths"] == REPLAY_EXCLUSIONS, "manifest closed replay exclusions drifted")
    require(replay["fresh_observations"] == {"records": [], "schema": "platform-plus-exact-pointer-value-records-v1", "transport": "temporary-actions-text-only-never-rewrites-accepted"}, "manifest fresh observation transport drifted")
    require(isinstance(manifest["platforms"], list) and len(manifest["platforms"]) == 2, "manifest must contain exactly two platforms")
    for index, platform_name in enumerate(PLATFORM_NAMES):
        validate_platform_record(manifest["platforms"][index], platform_name)
    if schema is not None:
        validate_against_schema(manifest, schema)
    reject_disallowed_claim_fields(manifest)


def reject_disallowed_claim_fields(value: Any, path: str = "manifest") -> None:
    if isinstance(value, list):
        for index, child in enumerate(value):
            reject_disallowed_claim_fields(child, f"{path}/{index}")
    elif isinstance(value, dict):
        denied = ("timing", "duration", "elapsed", "throughput", "speedup", "latency", "memory", "energy", "benchmark", "performance")
        for key, child in value.items():
            require(not any(word in key.lower() for word in denied), f"{path}/{key} is an unauthorized claim field")
            reject_disallowed_claim_fields(child, f"{path}/{key}")


def json_pointer_get(value: Any, pointer: str) -> Any:
    require(pointer.startswith("/"), f"invalid JSON pointer {pointer}")
    current = value
    for raw in pointer[1:].split("/"):
        token = raw.replace("~1", "/").replace("~0", "~")
        if isinstance(current, list):
            require(token.isdigit() and int(token) < len(current), f"JSON pointer does not resolve: {pointer}")
            current = current[int(token)]
        else:
            require(isinstance(current, dict) and token in current, f"JSON pointer does not resolve: {pointer}")
            current = current[token]
    return current


def json_pointer_set(value: Any, pointer: str, replacement: Any) -> None:
    tokens = pointer[1:].split("/")
    current = value
    for raw in tokens[:-1]:
        token = raw.replace("~1", "/").replace("~0", "~")
        current = current[int(token)] if isinstance(current, list) else current[token]
    final = tokens[-1].replace("~1", "/").replace("~0", "~")
    if isinstance(current, list):
        current[int(final)] = replacement
    else:
        current[final] = replacement


def replay_projection(manifest: Any) -> bytes:
    projected = json.loads(canonical_json_bytes(manifest))
    for pointer in REPLAY_EXCLUSIONS:
        json_pointer_set(projected, pointer, "<trace-only-observation>")
    return canonical_json_bytes(projected)


def pointer_platform(pointer: str) -> str:
    if pointer.startswith("/platforms/0/"):
        return PLATFORM_NAMES[0]
    if pointer.startswith("/platforms/1/"):
        return PLATFORM_NAMES[1]
    raise EvidenceError(f"replay pointer is outside the exact platform projection: {pointer}")


def build_fresh_observations(accepted_bytes: bytes, fresh_manifest: Any) -> dict[str, Any]:
    return {
        "accepted_manifest_sha256": sha256_bytes(accepted_bytes),
        "records": [
            {
                "platform": pointer_platform(pointer),
                "pointer": pointer,
                "value": json_pointer_get(fresh_manifest, pointer),
            }
            for pointer in REPLAY_EXCLUSIONS
        ],
        "schema_id": FRESH_OBSERVATIONS_ID,
    }


def validate_fresh_observations(value: Any, accepted_bytes: bytes, fresh_manifest: Any) -> None:
    root = exact_keys(value, {"accepted_manifest_sha256", "records", "schema_id"}, "fresh observations")
    require(root["schema_id"] == FRESH_OBSERVATIONS_ID, "fresh observation schema ID drifted")
    require(root["accepted_manifest_sha256"] == sha256_bytes(accepted_bytes), "fresh observations do not bind accepted manifest bytes")
    require(isinstance(root["records"], list) and len(root["records"]) == 48, "fresh observations must contain exactly 48 records")
    for index, pointer in enumerate(REPLAY_EXCLUSIONS):
        record = exact_keys(root["records"][index], {"platform", "pointer", "value"}, f"fresh observations[{index}]")
        require(record["platform"] == pointer_platform(pointer), f"fresh observations[{index}] platform drifted")
        require(record["pointer"] == pointer, f"fresh observations[{index}] pointer/order drifted")
        require(record["value"] == json_pointer_get(fresh_manifest, pointer), f"fresh observations[{index}] value does not match fresh manifest")


def validate_manifest_file_hashes(bundle: pathlib.Path, manifest: Mapping[str, Any]) -> None:
    paths = {
        "schema": REPOSITORY_ROOT / SCHEMA_PATH,
        "workflow": REPOSITORY_ROOT / WORKFLOW_PATH,
        "tool": REPOSITORY_ROOT / TOOL_PATH,
        "oracle": bundle / "oracle.json",
        "reproduce": bundle / "REPRODUCE.md",
    }
    for key, path in paths.items():
        require(path.is_file(), f"manifest-bound {key} file is missing: {path}")
        digest, _ = file_identity(path)
        require(digest == manifest[key]["sha256"], f"manifest.{key}.sha256 does not bind {path}")


REPRODUCE_REQUIRED = [
    SUBJECT_COMMIT,
    SUBJECT_TREE,
    CLAIM_ID,
    SCHEMA_ID,
    TOOL_ID,
    ORACLE_ID,
    TOOL_PATH,
    "linux-x86_64",
    "windows-x86_64",
    "runner image",
    "kernel",
    "not an immutable",
    "new corroboration",
    "target artifact",
    "observable result",
    "failure",
    "limitation",
    "no performance claim",
    "initial capture",
    "replay",
    "parsed Exit code: 91",
    "no application Output: or Error output:",
    "fresh observations never rewrite accepted observations",
    "canonical Git blob bytes",
    "separate detached Git checkout",
    "git status --porcelain=v1 --untracked-files=all",
    "CARGO_NET_OFFLINE true",
    "Native O0 and O2 exit 91 with empty stdout and stderr",
    "traceability-only replay exclusions",
    "equal SHA-256 and byte size",
    *ARTIFACT_NAMES,
    *COMMAND_NAMES,
    *REPLAY_EXCLUSIONS,
]


def validate_reproduce_bytes(payload: bytes) -> str:
    require(not payload.startswith(b"\xef\xbb\xbf"), "REPRODUCE.md has a UTF-8 BOM")
    try:
        text = payload.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise EvidenceError(f"REPRODUCE.md is not UTF-8: {exc}") from exc
    require(text.endswith("\n") and "\r" not in text, "REPRODUCE.md must be UTF-8/LF with a final LF")
    folded = text.casefold()
    for required in REPRODUCE_REQUIRED:
        require(required.casefold() in folded, f"REPRODUCE.md omitted frozen boundary {required!r}")
    return text


def validate_claim_index_value(index: Any) -> None:
    require(isinstance(index, dict) and isinstance(index.get("claims"), list), "claim index shape drifted")
    matches = [record for record in index["claims"] if isinstance(record, dict) and record.get("id") == CLAIM_ID]
    require(len(matches) == 1, "claim index must contain the CAP-024 claim exactly once")
    claim = matches[0]
    require(claim.get("status") == CLAIM_STATUS, "claim index status drifted")
    require(claim.get("artifacts") == [MANIFEST_PATH, ORACLE_PATH, REPRODUCE_PATH], "claim index artifact list drifted")
    prose = claim.get("claim")
    require(isinstance(prose, str) and SUBJECT_COMMIT in prose and "no performance claim" in prose.casefold(), "claim index boundary prose drifted")
    ids = [record.get("id") for record in index["claims"] if isinstance(record, dict)]
    require(len(ids) == len(set(ids)), "claim index contains duplicate IDs")


def validate_claim_index() -> None:
    path = REPOSITORY_ROOT / "claim-verification/claims.json"
    index = parse_json_bytes(path.read_bytes(), "claim index", canonical=False)
    validate_claim_index_value(index)


def validate_aggregate_staging(bundle: pathlib.Path, claim_index: Any | None = None) -> None:
    oracle = read_json(bundle / "oracle.json", "aggregate oracle")
    validate_oracle(oracle)
    validate_reproduce_bytes((bundle / "REPRODUCE.md").read_bytes())
    if claim_index is None:
        validate_claim_index()
    else:
        validate_claim_index_value(claim_index)


def validate_bundle_file_names(names: Iterable[str]) -> None:
    require(set(names) == REQUIRED_BUNDLE_FILES, "bundle must contain exactly REPRODUCE.md manifest.json oracle.json")


def validate_bundle(bundle: pathlib.Path) -> tuple[dict[str, Any], bytes]:
    require(bundle.is_dir(), f"bundle directory is missing: {bundle}")
    actual = {entry.name for entry in bundle.iterdir()}
    validate_bundle_file_names(actual)
    schema = read_json(REPOSITORY_ROOT / SCHEMA_PATH, "CAP-024 schema")
    validate_schema(schema)
    manifest_path = bundle / "manifest.json"
    manifest_bytes = manifest_path.read_bytes()
    manifest = parse_json_bytes(manifest_bytes, "accepted manifest", canonical=True)
    oracle = read_json(bundle / "oracle.json", "accepted oracle")
    validate_oracle(oracle)
    validate_reproduce_bytes((bundle / "REPRODUCE.md").read_bytes())
    validate_manifest(manifest, schema)
    validate_manifest_file_hashes(bundle, manifest)
    validate_claim_index()
    return manifest, manifest_bytes


def verify_embedded_support() -> None:
    require(len(LINUX_START) == LINUX_START_SIZE and sha256_bytes(LINUX_START) == LINUX_START_SHA256, "embedded linux-start.S identity drifted")
    require(len(WINDOWS_CHKSTK) == WINDOWS_CHKSTK_SIZE and sha256_bytes(WINDOWS_CHKSTK) == WINDOWS_CHKSTK_SHA256, "embedded windows-chkstk.S identity drifted")


def read_detached_head(subject: pathlib.Path) -> str:
    git = subject / ".git"
    if git.is_file():
        text = git.read_text(encoding="utf-8").strip()
        require(text.startswith("gitdir: "), "subject .git indirection is malformed")
        git = (subject / text.removeprefix("gitdir: ")).resolve()
    require(git.is_dir(), "subject checkout has no .git directory")
    head = (git / "HEAD").read_text(encoding="ascii").strip()
    require(valid_git_hash(head), "subject HEAD must be detached at the frozen commit")
    return head


def _git_object_hash(kind: str, payload: bytes) -> bytes:
    framed = f"{kind} {len(payload)}\0".encode("ascii") + payload
    return hashlib.sha1(framed, usedforsecurity=False).digest()


def _tree_hash(entries: Mapping[str, tuple[int, bytes]]) -> tuple[str, dict[str, str]]:
    tree: dict[str, Any] = {}
    for path, (mode, payload) in entries.items():
        parts = pathlib.PurePosixPath(path).parts
        require(parts and all(part not in {"", ".", ".."} for part in parts), f"subject path is invalid: {path}")
        node = tree
        for part in parts[:-1]:
            node = node.setdefault(part, {})
            require(isinstance(node, dict), f"subject path collision at {path}")
        node[parts[-1]] = (mode, _git_object_hash("blob", payload))
    hashes: dict[str, str] = {}

    def visit(node: dict[str, Any], prefix: str) -> bytes:
        encoded = bytearray()
        ordered = sorted(node.items(), key=lambda item: (item[0] + ("/" if isinstance(item[1], dict) else "")).encode("utf-8"))
        for name, child in ordered:
            if isinstance(child, dict):
                mode, digest = 0o40000, visit(child, f"{prefix}/{name}" if prefix else name)
            else:
                mode, digest = child
            encoded.extend(f"{mode:o} {name}".encode("utf-8"))
            encoded.append(0)
            encoded.extend(digest)
        digest = _git_object_hash("tree", bytes(encoded))
        hashes[prefix] = digest.hex()
        return digest

    root = visit(tree, "")
    return root.hex(), hashes


def _subject_snapshot(subject: pathlib.Path) -> dict[str, dict[str, Any]]:
    snapshot: dict[str, dict[str, Any]] = {}
    for path in sorted(subject.rglob("*")):
        relative = path.relative_to(subject).as_posix()
        if relative == ".git" or relative.startswith(".git/"):
            continue
        if path.is_symlink():
            payload = os.readlink(path).encode("utf-8")
            snapshot[relative] = {"kind": "symlink", "sha256": sha256_bytes(payload), "size": len(payload)}
        elif path.is_file():
            payload = path.read_bytes()
            snapshot[relative] = {"kind": "file", "sha256": sha256_bytes(payload), "size": len(payload)}
    return snapshot


def run_subject_git(subject: pathlib.Path, command_id: str) -> bytes:
    require(command_id in GIT_VERIFICATION_COMMANDS, f"unrecorded subject Git verification {command_id}")
    arguments = {
        "git:autocrlf": ["config", "--get", "core.autocrlf"],
        "git:compiler-tree": ["rev-parse", "--verify", "HEAD:src/compiler"],
        "git:ls-tree": ["ls-tree", "-rz", "HEAD"],
        "git:object-format": ["rev-parse", "--show-object-format"],
        "git:parents": ["show", "-s", "--format=%P", "HEAD"],
        "git:status": ["status", "--porcelain=v1", "--untracked-files=all"],
        "git:tree": ["rev-parse", "--verify", "HEAD^{tree}"],
    }[command_id]
    git = shutil.which("git")
    require(git is not None, "subject verification cannot locate Git")
    completed = subprocess.run(
        [git, "-C", str(subject), *arguments],
        cwd=subject,
        env=dict(os.environ),
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        shell=False,
        check=False,
    )
    require(
        completed.returncode == 0,
        f"{command_id} failed: {completed.stderr.decode('utf-8', errors='replace').strip()}",
    )
    require(completed.stderr == b"", f"{command_id} wrote unexpected stderr")
    return completed.stdout


def tracked_subject_entries(subject: pathlib.Path) -> dict[str, tuple[int, bytes]]:
    raw = run_subject_git(subject, "git:ls-tree")
    records = raw.split(b"\0")
    require(records and records[-1] == b"", "subject Git tree output is not NUL-terminated")
    entries: dict[str, tuple[int, bytes]] = {}
    for record in records[:-1]:
        try:
            metadata, raw_path = record.split(b"\t", 1)
            raw_mode, kind, raw_identity = metadata.split(b" ", 2)
            relative = raw_path.decode("utf-8")
            identity = raw_identity.decode("ascii")
            mode = int(raw_mode, 8)
        except (UnicodeError, ValueError) as exc:
            raise EvidenceError(f"subject Git tree record is malformed: {exc}") from exc
        require(
            kind == b"blob"
            and mode in {0o100644, 0o100755, 0o120000}
            and valid_git_hash(identity),
            f"subject Git tree entry is unsupported: {relative}",
        )
        require(relative not in entries, f"subject Git tree duplicates {relative}")
        path = subject.joinpath(*pathlib.PurePosixPath(relative).parts)
        if mode == 0o120000 and path.is_symlink():
            payload = os.readlink(path).encode("utf-8")
        else:
            require(path.is_file(), f"subject tracked file is missing: {relative}")
            payload = path.read_bytes()
        require(
            _git_object_hash("blob", payload).hex() == identity,
            f"subject tracked bytes differ from the index/HEAD blob: {relative}",
        )
        entries[relative] = (mode, payload)
    require(
        set(entries) == set(_subject_snapshot(subject)),
        "subject checkout contains missing or untracked files",
    )
    return entries


def extract_source_records(payload: bytes) -> list[list[int]]:
    try:
        source = payload.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise EvidenceError(f"frozen source is not UTF-8: {exc}") from exc
    names = [
        "ordinary_record",
        "wrapping_record",
        "activation_record",
        "tie_record",
        "malformed_first_record",
        "malformed_second_record",
        "malformed_third_record",
    ]
    result = []
    for name in names:
        match = re.search(rf"let\s+{re.escape(name)}\s*:\s*\[int;\s*20\]\s*=\s*\[([^]]+)\];", source)
        require(match is not None, f"frozen source omitted {name}")
        lanes = [int(piece.strip()) for piece in match.group(1).split(",") if piece.strip()]
        require(len(lanes) == 20, f"frozen source {name} does not have twenty lanes")
        result.append(lanes)
    return result


def verify_subject(subject: pathlib.Path) -> None:
    require(subject.is_dir(), f"subject checkout does not exist: {subject}")
    require(read_detached_head(subject) == SUBJECT_COMMIT, "subject checkout is not detached at the frozen accepted commit")
    require(
        run_subject_git(subject, "git:object-format").strip() == b"sha1",
        "subject checkout does not use the frozen SHA-1 Git object format",
    )
    require(
        run_subject_git(subject, "git:autocrlf").strip().lower() == b"false",
        "subject checkout does not disable line-ending conversion",
    )
    require(
        run_subject_git(subject, "git:status") == b"",
        "subject tracked/index state is not clean",
    )
    require(
        run_subject_git(subject, "git:tree").strip().decode("ascii") == SUBJECT_TREE,
        "subject HEAD tree drifted",
    )
    require(
        run_subject_git(subject, "git:compiler-tree").strip().decode("ascii")
        == COMPILER_TREE,
        "subject HEAD compiler tree drifted",
    )
    parents = run_subject_git(subject, "git:parents").decode("ascii").strip().split()
    require(parents == SUBJECT_PARENTS, "subject ordered parents drifted")
    tree, trees = _tree_hash(tracked_subject_entries(subject))
    require(tree == SUBJECT_TREE, "subject worktree bytes no longer match the accepted tree")
    require(trees.get("src/compiler") == COMPILER_TREE, "subject worktree compiler tree drifted")
    for frozen in FROZEN_INPUTS:
        path = subject / frozen["path"]
        require(path.is_file(), f"frozen subject input is missing: {frozen['path']}")
        payload = path.read_bytes()
        require(
            len(payload) == frozen["size"]
            and sha256_bytes(payload) == frozen["sha256"]
            and _git_object_hash("blob", payload).hex() == frozen["blob"],
            f"subject input differs from canonical Git blob bytes: {frozen['path']}",
        )
    source = (subject / FROZEN_INPUTS[0]["path"]).read_bytes()
    require(extract_source_records(source) == [record for _, record in SOURCE_RECORDS], "source literals differ from the seven frozen oracle records")


_ACTIVE_SUBSTITUTIONS: dict[str, pathlib.Path] = {}


def substitute_path(value: str, substitutions: Mapping[str, pathlib.Path] | None = None) -> str:
    result = value
    selected = _ACTIVE_SUBSTITUTIONS if substitutions is None else substitutions
    for token, path in selected.items():
        result = result.replace(f"${{{token}}}", str(path))
    return result


def materialize_command_env(env_spec: Mapping[str, Any], substitutions: Mapping[str, pathlib.Path] | None = None) -> dict[str, str]:
    inheritance = env_spec["inheritance"]
    require(inheritance in {"none", "runner-substrate-observation-only"}, "unknown command environment inheritance policy")
    environment = dict(os.environ) if inheritance == "runner-substrate-observation-only" else {}
    for key, value in env_spec["overrides"].items():
        environment[key] = substitute_path(value, substitutions)
    prefix = [substitute_path(value, substitutions) for value in env_spec["path_prefix"]]
    inherited_path = environment.get("PATH", "") if inheritance != "none" else ""
    environment["PATH"] = os.pathsep.join(prefix + ([inherited_path] if inherited_path else []))
    return environment


def materialize_argv(argv: Sequence[str]) -> list[str]:
    return [substitute_path(value) for value in argv]


def run_recorded_subprocess(command_id: str, command_spec: Mapping[str, Any]) -> dict[str, Any]:
    require(command_id in PIPELINE_COMMANDS, f"unrecorded pipeline subprocess {command_id}")
    argv = materialize_argv(command_spec["argv"])
    require(argv and argv[0] != "internal", f"internal command {command_id} cannot use subprocess")
    for produced in command_spec["produces"]:
        pathlib.Path(substitute_path(produced)).parent.mkdir(parents=True, exist_ok=True)
    completed = subprocess.run(
        argv,
        cwd=substitute_path(command_spec["cwd"]),
        env=materialize_command_env(command_spec["env"]),
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        shell=False,
        check=False,
    )
    record = dict(command_spec)
    record["stdout"] = byte_record(completed.stdout)
    record["stderr"] = byte_record(completed.stderr)
    record["exit_code"] = completed.returncode
    require(completed.returncode == command_spec["exit_code"], f"{command_id} exited {completed.returncode}, expected {command_spec['exit_code']}")
    for produced in command_spec["produces"]:
        require(pathlib.Path(substitute_path(produced)).is_file(), f"{command_id} did not produce {produced}")
    return record


def run_tool_version_probe(tool_id: str, platform_name: str) -> dict[str, Any]:
    command_id = f"version:{tool_id}"
    require(command_id in TOOL_VERSION_PROBES, f"unrecorded tool-version subprocess {command_id}")
    normalized_path = expected_tool_path(platform_name, tool_id)
    path = pathlib.Path(substitute_path(normalized_path))
    require(path.is_file(), f"pinned tool payload is missing: {path}")
    argv = [str(path), "-Vv" if tool_id in {"cargo", "rustc"} else "--version"]
    completed = subprocess.run(
        argv,
        cwd=substitute_path("${SUBJECT}"),
        env=materialize_command_env(command_environment(platform_name, "clean_before")),
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        shell=False,
        check=False,
    )
    require(completed.returncode == 0, f"{tool_id} version probe failed")
    parsed = parse_tool_banner(tool_id, completed.stdout, completed.stderr)
    digest, size = file_identity(path)
    return {
        "path": normalized_path,
        "payload_sha256": digest,
        "payload_size": size,
        "version": {
            "argv": [normalized_path, argv[1]],
            "exit_code": 0,
            "parsed": parsed,
            "stderr": byte_record(completed.stderr),
            "stdout": byte_record(completed.stdout),
        },
    }


def internal_command_record(platform_name: str, name: str) -> dict[str, Any]:
    spec = expected_command_spec(platform_name, name)
    return {**spec, "stderr": byte_record(b""), "stdout": byte_record(b"")}


def platform_observations(platform_name: str) -> dict[str, str]:
    image_parts = [os.environ.get("ImageOS", ""), os.environ.get("ImageVersion", "")]
    runner_image = "/".join(part for part in image_parts if part) or "unversioned-compatible-runner"
    if platform_name == "linux-x86_64" and hasattr(os, "uname"):
        identity = os.uname()
        kernel = f"{identity.sysname} {identity.release} {identity.machine}"
    elif platform_name == "windows-x86_64" and hasattr(sys, "getwindowsversion"):
        version = sys.getwindowsversion()
        kernel = f"Windows {version.major}.{version.minor}.{version.build} {host_platform.machine()}"
    else:
        kernel = f"{sys.platform} {host_platform.machine()}"
    return {"kernel": kernel, "runner_image": runner_image}


def acquire_llvm_archive(platform_name: str, destination: pathlib.Path) -> pathlib.Path:
    archive = platform_archive(platform_name)
    destination.mkdir(parents=True, exist_ok=True)
    target = destination / archive["name"]
    if not target.is_file():
        encoded_name = archive["name"].replace("+", "%2B")
        url = f"https://github.com/llvm/llvm-project/releases/download/llvmorg-22.1.8/{encoded_name}"
        temporary = target.with_suffix(target.suffix + ".partial")
        with urllib.request.urlopen(url) as response, temporary.open("wb") as output:
            shutil.copyfileobj(response, output)
        temporary.replace(target)
    verify_archive(platform_name, target)
    return target


def verify_archive(platform_name: str, path: pathlib.Path) -> None:
    archive = platform_archive(platform_name)
    require(path.is_file(), f"pinned LLVM archive is missing: {path}")
    digest, size = file_identity(path)
    require(size == archive["size"], f"{platform_name} LLVM archive size drifted")
    require(digest == archive["sha256"], f"{platform_name} LLVM archive digest drifted")


def extract_archive(path: pathlib.Path, destination: pathlib.Path) -> pathlib.Path:
    destination = destination.resolve()
    require(destination.name.startswith("llvm-"), "LLVM extraction destination is not task-local")
    if destination.exists():
        shutil.rmtree(destination)
    destination.mkdir(parents=True)
    root = destination.resolve()
    with tarfile.open(path, mode="r:xz") as archive:
        for member in archive.getmembers():
            resolved = (destination / member.name).resolve()
            require(resolved == root or root in resolved.parents, f"LLVM archive member escapes extraction root: {member.name}")
        archive.extractall(destination, filter="data")
    candidates = [entry for entry in destination.iterdir() if entry.is_dir() and (entry / "bin").is_dir()]
    if (destination / "bin").is_dir():
        return destination.resolve()
    require(len(candidates) == 1, "LLVM archive extraction did not yield one tool root")
    return candidates[0].resolve()


def locked_registry_packages(
    lock_path: pathlib.Path, expected_registry_count: int = 112
) -> list[dict[str, str]]:
    """Return the exact checksummed crates.io packages frozen by Cargo.lock."""

    try:
        lock = tomllib.loads(lock_path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, tomllib.TOMLDecodeError) as exc:
        raise EvidenceError(f"cannot parse canonical Cargo.lock: {exc}") from exc
    require(lock.get("version") == 4, "canonical Cargo.lock must use format version 4")
    packages = lock.get("package")
    require(isinstance(packages, list), "canonical Cargo.lock package list is missing")
    registry: list[dict[str, str]] = []
    local: list[Mapping[str, Any]] = []
    identities: set[tuple[str, str]] = set()
    for package in packages:
        require(isinstance(package, dict), "canonical Cargo.lock package record is malformed")
        name = package.get("name")
        version = package.get("version")
        source = package.get("source")
        checksum = package.get("checksum")
        require(
            isinstance(name, str)
            and isinstance(version, str)
            and re.fullmatch(r"[A-Za-z0-9_.+-]+", name) is not None
            and re.fullmatch(r"[A-Za-z0-9_.+-]+", version) is not None,
            "canonical Cargo.lock contains an unsafe package identity",
        )
        identity = (name, version)
        require(identity not in identities, f"canonical Cargo.lock duplicates {name} {version}")
        identities.add(identity)
        if source is None:
            require(checksum is None, f"local package {name} unexpectedly carries a checksum")
            local.append(package)
            continue
        require(source == CRATES_IO_SOURCE, f"Cargo.lock package {name} uses an unauthorized source")
        require(isinstance(checksum, str) and valid_sha256(checksum), f"Cargo.lock package {name} has no exact SHA-256 checksum")
        registry.append({"checksum": checksum, "name": name, "version": version})
    require(
        len(local) == 1
        and local[0].get("name") == "compiler"
        and local[0].get("version") == "0.3.0",
        "canonical Cargo.lock local package topology drifted",
    )
    require(
        len(registry) == expected_registry_count,
        "canonical Cargo.lock registry package topology drifted",
    )
    return sorted(registry, key=lambda package: (package["name"], package["version"]))


def acquire_locked_crate(package: Mapping[str, str], cache: pathlib.Path) -> pathlib.Path:
    cache.mkdir(parents=True, exist_ok=True)
    filename = f"{package['name']}-{package['version']}.crate"
    destination = cache / filename
    if destination.is_file():
        digest, size = file_identity(destination)
        require(size > 0 and digest == package["checksum"], f"cached locked crate {filename} failed checksum verification")
        return destination
    temporary = cache / f".{filename}.partial"
    if temporary.exists():
        temporary.unlink()
    url = f"{CRATES_IO_DOWNLOAD_PREFIX}{package['name']}/{filename}"
    request = urllib.request.Request(url, headers={"User-Agent": "Aero-CAP024/1"})
    with urllib.request.urlopen(request) as response, temporary.open("xb") as output:
        shutil.copyfileobj(response, output)
    digest, size = file_identity(temporary)
    require(size > 0 and digest == package["checksum"], f"downloaded locked crate {filename} failed checksum verification")
    temporary.replace(destination)
    return destination


def extract_locked_crate(package: Mapping[str, str], archive_path: pathlib.Path, vendor: pathlib.Path) -> None:
    package_root_name = f"{package['name']}-{package['version']}"
    destination = (vendor / package_root_name).resolve()
    require(destination.parent == vendor.resolve(), f"locked crate destination escaped vendor root: {package_root_name}")
    destination.mkdir(parents=True, exist_ok=False)
    file_hashes: dict[str, str] = {}
    portable_paths: set[str] = set()
    with tarfile.open(archive_path, mode="r:gz") as archive:
        for member in archive.getmembers():
            parts = pathlib.PurePosixPath(member.name).parts
            require(parts and parts[0] == package_root_name, f"locked crate {package_root_name} has an unexpected archive root")
            if len(parts) == 1 or member.isdir():
                continue
            relative_parts = parts[1:]
            require(
                all(
                    part not in {"", ".", ".."}
                    and "\\" not in part
                    and ":" not in part
                    for part in relative_parts
                ),
                f"locked crate {package_root_name} has an unsafe path",
            )
            require(member.isfile(), f"locked crate {package_root_name} contains a non-file member")
            relative = pathlib.PurePosixPath(*relative_parts).as_posix()
            portable = relative.casefold()
            require(
                portable != ".cargo-checksum.json" and portable not in portable_paths,
                f"locked crate {package_root_name} duplicates a portable path",
            )
            portable_paths.add(portable)
            stream = archive.extractfile(member)
            require(stream is not None, f"locked crate {package_root_name} member cannot be read")
            payload = stream.read()
            target = destination.joinpath(*relative_parts)
            resolved = target.resolve()
            require(resolved == destination or destination in resolved.parents, f"locked crate {package_root_name} escapes its destination")
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(payload)
            if member.mode & 0o111 and os.name != "nt":
                target.chmod(0o755)
            file_hashes[relative] = sha256_bytes(payload)
    require("Cargo.toml" in file_hashes, f"locked crate {package_root_name} omitted Cargo.toml")
    checksum_record = {"files": file_hashes, "package": package["checksum"]}
    (destination / ".cargo-checksum.json").write_bytes(canonical_json_bytes(checksum_record))


def materialize_locked_vendor(
    lock_path: pathlib.Path,
    platform_root: pathlib.Path,
    cache: pathlib.Path,
    expected_registry_count: int = 112,
) -> None:
    """Create a lockfile-complete Cargo vendor before any offline build."""

    packages = locked_registry_packages(lock_path, expected_registry_count)
    vendor = (platform_root / "cargo-vendor").resolve()
    cargo_home = (platform_root / "cargo-home").resolve()
    require(vendor.parent == platform_root.resolve() and cargo_home.parent == platform_root.resolve(), "locked vendor path escaped platform work root")
    vendor.mkdir(parents=True, exist_ok=False)
    cargo_home.mkdir(parents=True, exist_ok=False)
    for package in packages:
        archive_path = acquire_locked_crate(package, cache)
        extract_locked_crate(package, archive_path, vendor)
    materialized = sorted(entry.name for entry in vendor.iterdir() if entry.is_dir())
    expected = sorted(f"{package['name']}-{package['version']}" for package in packages)
    require(materialized == expected, LOCKFILE_VENDOR_BOUNDARY)
    config = (
        "[net]\n"
        "offline = true\n\n"
        "[source.crates-io]\n"
        'replace-with = "vendored-sources"\n\n'
        "[source.vendored-sources]\n"
        f"directory = {json.dumps(vendor.as_posix())}\n"
    )
    (cargo_home / "config.toml").write_text(config, encoding="utf-8", newline="\n")


def discover_rust_root() -> pathlib.Path:
    """Select the installed toolchain payload, never a floating launcher shim."""

    configured = os.environ.get("CAP024_RUST_ROOT")
    if configured:
        return pathlib.Path(configured).resolve()

    manager_home_value = os.environ.get("RUST" + "UP_HOME")
    manager_home = (
        pathlib.Path(manager_home_value).resolve()
        if manager_home_value
        else pathlib.Path.home() / (".rust" + "up")
    )
    toolchains = manager_home / "toolchains"
    requested: list[str] = []
    selected = os.environ.get("RUST" + "UP_TOOLCHAIN")
    if selected and (selected == RUST_VERSION or selected.startswith(f"{RUST_VERSION}-")):
        requested.append(selected)
    installed: list[pathlib.Path] = []
    if toolchains.is_dir():
        installed = sorted((entry for entry in toolchains.iterdir() if entry.is_dir()), key=lambda entry: entry.name)
        requested.extend(entry.name for entry in installed if entry.name.startswith(f"{RUST_VERSION}-"))
    settings = manager_home / "settings.toml"
    if settings.is_file():
        match = re.search(
            r'^default_toolchain\s*=\s*"([^"]+)"\s*$',
            settings.read_text(encoding="utf-8"),
            flags=re.MULTILINE,
        )
        if match and (match.group(1) == RUST_VERSION or match.group(1).startswith(f"{RUST_VERSION}-")):
            requested.append(match.group(1))
    if toolchains.is_dir():
        for name in dict.fromkeys(requested):
            candidate = (toolchains / name).resolve()
            suffix = ".exe" if os.name == "nt" else ""
            if (candidate / "bin" / f"cargo{suffix}").is_file() and (candidate / "bin" / f"rustc{suffix}").is_file():
                return candidate

    cargo = shutil.which("cargo")
    rustc = shutil.which("rustc")
    require(cargo is not None and rustc is not None, "capture cannot locate pinned cargo and rustc")
    cargo_root = pathlib.Path(cargo).resolve().parent.parent
    rustc_root = pathlib.Path(rustc).resolve().parent.parent
    require(cargo_root == rustc_root, "cargo and rustc do not share one explicit Rust root")
    return cargo_root


def resolve_capture_paths(args: argparse.Namespace) -> tuple[pathlib.Path, pathlib.Path, pathlib.Path, pathlib.Path, pathlib.Path]:
    transport = pathlib.Path(os.environ.get(TRANSPORT_ENV, REPOSITORY_ROOT / "cap024-evidence-transport")).resolve()
    subject = pathlib.Path(args.subject_dir).resolve() if args.subject_dir else transport / "_workspace" / "subject"
    work = pathlib.Path(args.work_dir).resolve() if args.work_dir else transport / "_workspace" / "work"
    require(subject != work and subject not in work.parents and work not in subject.parents, "subject and work roots must be disjoint")
    if not args.subject_dir:
        require(subject.parent == transport / "_workspace", "managed subject path escaped the CAP-024 transport workspace")
    require((subject / ".git").exists(), "capture requires a separate Git checkout of the frozen subject")
    if args.rust_dir:
        rust = pathlib.Path(args.rust_dir).resolve()
    else:
        rust = discover_rust_root()
    if args.llvm_archive:
        archive = pathlib.Path(args.llvm_archive).resolve()
        verify_archive(args.platform, archive)
    else:
        acquire_root = pathlib.Path(args.acquire_directory).resolve() if args.acquire_directory else transport / "_downloads"
        archive = acquire_llvm_archive(args.platform, acquire_root)
    if args.llvm_dir:
        llvm = pathlib.Path(args.llvm_dir).resolve()
    else:
        llvm = extract_archive(archive, transport / "_tools" / f"llvm-{args.platform}")
    require((rust / "bin").is_dir(), f"Rust root has no bin directory: {rust}")
    require((llvm / "bin").is_dir(), f"LLVM root has no bin directory: {llvm}")
    return subject, work, llvm, rust, archive


def safely_reset_platform_work(work: pathlib.Path, platform_name: str) -> None:
    work.mkdir(parents=True, exist_ok=True)
    root = work.resolve()
    target = (root / platform_name).resolve()
    require(target.parent == root and target != root, "unsafe CAP-024 work target")
    if target.exists():
        shutil.rmtree(target)
    target.mkdir(parents=True)
    (root / "tmp").mkdir(exist_ok=True)


def write_launch_support(work: pathlib.Path, platform_name: str) -> None:
    if platform_name == "linux-x86_64":
        path, payload = work / platform_name / "linux-start.S", LINUX_START
    else:
        path, payload = work / platform_name / "windows-chkstk.S", WINDOWS_CHKSTK
    path.write_bytes(payload)


def normalized_file_record(path_value: str, producer_command: str) -> dict[str, Any]:
    path = pathlib.Path(substitute_path(path_value))
    require(path.is_file(), f"recorded product is missing: {path}")
    digest, size = file_identity(path)
    require(size > 0, f"recorded product is empty: {path}")
    return {"path": path_value, "producer_command": producer_command, "sha256": digest, "size": size}


def capture_platform(args: argparse.Namespace) -> dict[str, Any]:
    require(args.platform in PLATFORM_NAMES, "capture requires an exact supported --platform")
    subject, work, llvm, rust, archive_path = resolve_capture_paths(args)
    global _ACTIVE_SUBSTITUTIONS
    _ACTIVE_SUBSTITUTIONS = {"SUBJECT": subject, "WORK": work, "LLVM": llvm, "RUST": rust}
    verify_embedded_support()
    verify_subject(subject)
    clean_before = internal_command_record(args.platform, "clean_before")
    safely_reset_platform_work(work, args.platform)
    transport = pathlib.Path(os.environ.get(TRANSPORT_ENV, REPOSITORY_ROOT / "cap024-evidence-transport")).resolve()
    dependency_cache_root = (
        pathlib.Path(args.acquire_directory).resolve()
        if args.acquire_directory
        else transport / "_downloads"
    )
    materialize_locked_vendor(
        subject / "src/compiler/Cargo.lock",
        work / args.platform,
        dependency_cache_root / "crates",
    )
    write_launch_support(work, args.platform)

    tools = {tool: run_tool_version_probe(tool, args.platform) for tool in TOOL_NAMES}
    commands: dict[str, Any] = {"clean_before": clean_before}
    for name in COMMAND_NAMES[1:-1]:
        spec = expected_command_spec(args.platform, name)
        commands[name] = run_recorded_subprocess(name, spec)
    verify_subject(subject)
    commands["clean_after"] = internal_command_record(args.platform, "clean_after")

    compiler_executables = {
        production: normalized_file_record(
            compiler_path(args.platform, production), f"compiler_build_{production}"
        )
        for production in ("first", "second")
    }
    artifacts: dict[str, Any] = {}
    for artifact in ARTIFACT_NAMES:
        first = normalized_file_record(
            artifact_path(args.platform, "first", artifact), artifact_producer(artifact, "first")
        )
        second = normalized_file_record(
            artifact_path(args.platform, "second", artifact), artifact_producer(artifact, "second")
        )
        require(first["sha256"] == second["sha256"] and first["size"] == second["size"], f"{args.platform} {artifact} productions are not byte-identical")
        artifacts[artifact] = {"first": first, "pair_equal": True, "second": second}
    archive = platform_archive(args.platform)
    verify_archive(args.platform, archive_path)
    platform_record = {
        "artifacts": artifacts,
        "commands": commands,
        "compiler_executables": compiler_executables,
        "failures": [],
        "name": args.platform,
        "observations": platform_observations(args.platform),
        "public_semantics": derive_public_semantics(commands, args.platform),
        "toolchain": {
            "archive_name": archive["name"],
            "archive_sha256": archive["sha256"],
            "archive_size": archive["size"],
            "llvm_version": LLVM_VERSION,
            "rust_commit": RUST_COMMIT,
            "rust_version": RUST_VERSION,
            "setup_boundary": "workflow-acquisition-only; every final tool payload and version is verified before capture",
            "tools": tools,
        },
    }
    validate_platform_record(platform_record, args.platform)
    return {
        "platform": platform_record,
        "schema_id": "cap024-platform-capture-v1",
        "subject_commit": SUBJECT_COMMIT,
        "tool_id": TOOL_ID,
    }


def validate_capture_record(value: Any, expected_platform: str | None = None) -> dict[str, Any]:
    root = exact_keys(value, {"platform", "schema_id", "subject_commit", "tool_id"}, "platform capture")
    require(root["schema_id"] == "cap024-platform-capture-v1", "platform capture schema drifted")
    require(root["subject_commit"] == SUBJECT_COMMIT and root["tool_id"] == TOOL_ID, "platform capture subject/tool drifted")
    name = root["platform"].get("name") if isinstance(root["platform"], dict) else None
    require(name in PLATFORM_NAMES, "platform capture name is invalid")
    if expected_platform is not None:
        require(name == expected_platform, f"expected {expected_platform} capture, received {name}")
    validate_platform_record(root["platform"], name)
    return root["platform"]


def reference_hash(path: pathlib.Path) -> str:
    require(path.is_file(), f"manifest reference file is missing: {path}")
    return sha256_bytes(path.read_bytes())


def build_manifest(linux: dict[str, Any], windows: dict[str, Any], bundle: pathlib.Path) -> dict[str, Any]:
    manifest = {
        "authorization_head": AUTHORIZATION_HEAD,
        "claim_id": CLAIM_ID,
        "failures": [],
        "inputs": FROZEN_INPUTS,
        "limitations": LIMITATIONS,
        "oracle": {"path": ORACLE_PATH, "sha256": reference_hash(bundle / "oracle.json")},
        "platforms": [linux, windows],
        "replay": {
            "canonical_projection": "sorted-compact-json-plus-lf-v1",
            "excluded_paths": REPLAY_EXCLUSIONS,
            "fresh_observations": {
                "records": [],
                "schema": "platform-plus-exact-pointer-value-records-v1",
                "transport": "temporary-actions-text-only-never-rewrites-accepted",
            },
        },
        "reproduce": {"path": REPRODUCE_PATH, "sha256": reference_hash(bundle / "REPRODUCE.md")},
        "schema": {"path": SCHEMA_PATH, "sha256": reference_hash(REPOSITORY_ROOT / SCHEMA_PATH)},
        "schema_id": SCHEMA_ID,
        "schema_version": 1,
        "scope": ALLOWED_PATHS,
        "subject": {
            "clean_after": True,
            "clean_before": True,
            "commit": SUBJECT_COMMIT,
            "compiler_tree": COMPILER_TREE,
            "parents": SUBJECT_PARENTS,
            "tree": SUBJECT_TREE,
        },
        "support": {
            "linux": {"path": "linux-start.S", "sha256": LINUX_START_SHA256, "size": LINUX_START_SIZE},
            "windows": {"path": "windows-chkstk.S", "sha256": WINDOWS_CHKSTK_SHA256, "size": WINDOWS_CHKSTK_SIZE},
        },
        "tool": {"id": TOOL_ID, "path": TOOL_PATH, "sha256": reference_hash(REPOSITORY_ROOT / TOOL_PATH)},
        "transport": "temporary-actions-text-only",
        "workflow": {"path": WORKFLOW_PATH, "sha256": reference_hash(REPOSITORY_ROOT / WORKFLOW_PATH)},
    }
    schema = read_json(REPOSITORY_ROOT / SCHEMA_PATH, "CAP-024 schema")
    validate_schema(schema)
    validate_manifest(manifest, schema)
    return manifest


def validate_failure_record(value: Any, expected_mode: str, expected_platform: str | None) -> str:
    root = exact_keys(value, {"failure", "schema_id"}, "canonical failure record")
    require(root["schema_id"] == FAILURE_SCHEMA_ID, "canonical failure schema ID drifted")
    failure = exact_keys(root["failure"], {"message", "mode", "platform"}, "canonical failure payload")
    require(failure["mode"] == expected_mode and failure["platform"] == expected_platform, "canonical failure routing drifted")
    require(isinstance(failure["message"], str) and failure["message"], "canonical failure message is empty")
    return failure["message"]


def aggregate_records(args: argparse.Namespace) -> dict[str, Any]:
    transport = pathlib.Path(os.environ.get(TRANSPORT_ENV, REPOSITORY_ROOT / "cap024-evidence-transport")).resolve()
    bundle = pathlib.Path(args.bundle).resolve() if args.bundle else REPOSITORY_ROOT / BUNDLE_DIRECTORY
    failures: list[str] = []
    if not bundle.is_dir():
        failures.append("aggregate staging directory is missing")
    else:
        try:
            actual_without_manifest = {
                entry.name for entry in bundle.iterdir() if entry.name != "manifest.json"
            }
        except OSError as exc:
            failures.append(f"aggregate staging directory cannot be read: {exc}")
        else:
            if actual_without_manifest != {"REPRODUCE.md", "oracle.json"}:
                failures.append(
                    "aggregate staging bundle files invalid: expected only REPRODUCE.md and oracle.json, "
                    f"received {sorted(actual_without_manifest)}"
                )
            else:
                try:
                    validate_aggregate_staging(bundle)
                except EvidenceError as exc:
                    failures.append(f"aggregate staging contract invalid: {exc}")
    configured_paths = {
        "linux-x86_64": args.linux_record or args.linux or transport / "linux-x86_64" / "capture.json",
        "windows-x86_64": args.windows_record or args.windows or transport / "windows-x86_64" / "capture.json",
    }
    job_results = {
        "linux-x86_64": os.environ.get(LINUX_JOB_RESULT_ENV),
        "windows-x86_64": os.environ.get(WINDOWS_JOB_RESULT_ENV),
    }
    captures: dict[str, dict[str, Any]] = {}
    for platform_name in PLATFORM_NAMES:
        result = job_results[platform_name]
        capture_path = pathlib.Path(configured_paths[platform_name])
        failure_path = capture_path.with_name("failure.json")
        if result == "success":
            if failure_path.exists():
                try:
                    message = validate_failure_record(read_json(failure_path, f"{platform_name} failure"), "capture", platform_name)
                except EvidenceError as exc:
                    message = str(exc)
                failures.append(f"{platform_name} reported success but transported failure: {message}")
                continue
            try:
                captures[platform_name] = validate_capture_record(read_json(capture_path, f"{platform_name} capture"), platform_name)
            except EvidenceError as exc:
                failures.append(f"{platform_name} success record invalid: {exc}")
        else:
            if result is None:
                failures.append(f"{platform_name} job result environment is missing")
            else:
                failures.append(f"{platform_name} capture job result was {result}")
            try:
                message = validate_failure_record(read_json(failure_path, f"{platform_name} failure"), "capture", platform_name)
                failures.append(f"{platform_name} captured failure: {message}")
            except EvidenceError as exc:
                failures.append(f"{platform_name} failure transport invalid: {exc}")
    require(not failures, "aggregate inspected both capture outcomes and failed closed: " + " | ".join(failures))
    linux = captures["linux-x86_64"]
    windows = captures["windows-x86_64"]
    manifest = build_manifest(linux, windows, bundle)
    validate_manifest_file_hashes(bundle, manifest)
    aggregate = transport / "aggregate"
    aggregate.mkdir(parents=True, exist_ok=True)
    output = pathlib.Path(args.output).resolve() if args.output else aggregate / "manifest.json"
    write_json(output, manifest)
    shutil.copyfile(bundle / "oracle.json", aggregate / "oracle.json")
    shutil.copyfile(bundle / "REPRODUCE.md", aggregate / "REPRODUCE.md")
    return manifest


NEGATIVE_CASES = {
    "schema_unknown",
    "schema_float",
    "schema_duplicate",
    "oracle_drift",
    "aggregate_oracle_drift",
    "aggregate_reproduce_drift",
    "aggregate_claim_index_drift",
    "aggregate_manifest_hash_drift",
    "artifact_pair",
    "artifact_path",
    "artifact_producer",
    "command_argv",
    "command_env",
    "public_missing",
    "public_duplicate",
    "public_wrong_exit",
    "public_output",
    "public_error",
    "public_prefixed_exit",
    "public_exit_in_stderr",
    "included_replay_difference",
    "excluded_replay_leaves",
    "extra_bundle_file",
    "reproduce_drift",
    "accepted_manifest_immutable",
}
INTERFACE_CONTRACT = "--mode self-test | --mode validate --bundle | --mode replay --bundle"
FAILURE_SCHEMA_ID = "cap024-canonical-failure-record-v1"
FAILURE_TRANSPORT = "capture exceptions become failure records"
CARGO_FIXTURE_COMMIT = "c980f4866141969fab6254a680546a277789d6f0"


def clone_json(value: Any) -> Any:
    return json.loads(canonical_json_bytes(value))


def expect_evidence_failure(action: Any, label: str) -> None:
    try:
        action()
    except EvidenceError:
        return
    raise EvidenceError(f"{label} mutation unexpectedly passed")


def fixture_tool_stdout(tool: str) -> bytes:
    if tool == "rustc":
        return (
            f"rustc {RUST_VERSION} ({RUST_COMMIT[:9]} 2026-07-14)\n"
            "binary: rustc\n"
            f"commit-hash: {RUST_COMMIT}\n"
            "commit-date: 2026-07-14\n"
            "host: x86_64-unknown-fixture\n"
            f"release: {RUST_VERSION}\n"
            "LLVM version: 22.1.6\n"
        ).encode("utf-8")
    if tool == "cargo":
        return (
            f"cargo {RUST_VERSION} ({CARGO_FIXTURE_COMMIT[:9]} 2026-06-30)\n"
            f"release: {RUST_VERSION}\n"
            f"commit-hash: {CARGO_FIXTURE_COMMIT}\n"
        ).encode("utf-8")
    if tool == "clang":
        return f"{CLANG_VERSION_BANNER}\nTarget: x86_64-fixture\nInstalledDir: /fixture/bin\n".encode("utf-8")
    if tool == "lld":
        return f"{LLD_VERSION_BANNER}\n".encode("utf-8")
    return f"fixture tool\n  {LLVM_VERSION_BANNER}\n".encode("utf-8")


def fixture_tool_record(platform_name: str, tool: str) -> dict[str, Any]:
    path = expected_tool_path(platform_name, tool)
    stdout = fixture_tool_stdout(tool)
    return {
        "path": path,
        "payload_sha256": sha256_bytes(f"{platform_name}:{tool}:payload".encode("utf-8")),
        "payload_size": len(platform_name) + len(tool) + 100,
        "version": {
            "argv": [path, "-Vv" if tool in {"cargo", "rustc"} else "--version"],
            "exit_code": 0,
            "parsed": parse_tool_banner(tool, stdout, b""),
            "stderr": byte_record(b""),
            "stdout": byte_record(stdout),
        },
    }


def fixture_command_record(platform_name: str, name: str) -> dict[str, Any]:
    spec = expected_command_spec(platform_name, name)
    if name == "public_run":
        stdout = b"Aero execution diagnostic\nExit code: 91\n"
    elif name in {"aero_build_llvm_first", "aero_build_llvm_second"}:
        stdout = b"diagnostic\n"
    else:
        stdout = b""
    return {**spec, "stderr": byte_record(b""), "stdout": byte_record(stdout)}


def fixture_platform_record(platform_name: str) -> dict[str, Any]:
    commands = {name: fixture_command_record(platform_name, name) for name in COMMAND_NAMES}
    artifacts: dict[str, Any] = {}
    for artifact in ARTIFACT_NAMES:
        digest = sha256_bytes(f"{platform_name}:{artifact}:pair".encode("utf-8"))
        size = len(platform_name) + len(artifact) + 200
        artifacts[artifact] = {
            "first": {
                "path": artifact_path(platform_name, "first", artifact),
                "producer_command": artifact_producer(artifact, "first"),
                "sha256": digest,
                "size": size,
            },
            "pair_equal": True,
            "second": {
                "path": artifact_path(platform_name, "second", artifact),
                "producer_command": artifact_producer(artifact, "second"),
                "sha256": digest,
                "size": size,
            },
        }
    compiler_executables = {
        production: {
            "path": compiler_path(platform_name, production),
            "producer_command": f"compiler_build_{production}",
            "sha256": sha256_bytes(f"{platform_name}:compiler:{production}".encode("utf-8")),
            "size": len(platform_name) + len(production) + 1000,
        }
        for production in ("first", "second")
    }
    archive = platform_archive(platform_name)
    return {
        "artifacts": artifacts,
        "commands": commands,
        "compiler_executables": compiler_executables,
        "failures": [],
        "name": platform_name,
        "observations": {"kernel": f"{platform_name}-kernel", "runner_image": f"{platform_name}-runner"},
        "public_semantics": derive_public_semantics(commands, platform_name),
        "toolchain": {
            "archive_name": archive["name"],
            "archive_sha256": archive["sha256"],
            "archive_size": archive["size"],
            "llvm_version": LLVM_VERSION,
            "rust_commit": RUST_COMMIT,
            "rust_version": RUST_VERSION,
            "setup_boundary": "workflow-acquisition-only; every final tool payload and version is verified before capture",
            "tools": {tool: fixture_tool_record(platform_name, tool) for tool in TOOL_NAMES},
        },
    }


def fixture_manifest() -> dict[str, Any]:
    reference = lambda name: sha256_bytes(f"fixture:{name}".encode("utf-8"))
    return {
        "authorization_head": AUTHORIZATION_HEAD,
        "claim_id": CLAIM_ID,
        "failures": [],
        "inputs": clone_json(FROZEN_INPUTS),
        "limitations": list(LIMITATIONS),
        "oracle": {"path": ORACLE_PATH, "sha256": reference("oracle")},
        "platforms": [fixture_platform_record(name) for name in PLATFORM_NAMES],
        "replay": {
            "canonical_projection": "sorted-compact-json-plus-lf-v1",
            "excluded_paths": list(REPLAY_EXCLUSIONS),
            "fresh_observations": {
                "records": [],
                "schema": "platform-plus-exact-pointer-value-records-v1",
                "transport": "temporary-actions-text-only-never-rewrites-accepted",
            },
        },
        "reproduce": {"path": REPRODUCE_PATH, "sha256": reference("reproduce")},
        "schema": {"path": SCHEMA_PATH, "sha256": reference("schema")},
        "schema_id": SCHEMA_ID,
        "schema_version": 1,
        "scope": list(ALLOWED_PATHS),
        "subject": {
            "clean_after": True,
            "clean_before": True,
            "commit": SUBJECT_COMMIT,
            "compiler_tree": COMPILER_TREE,
            "parents": list(SUBJECT_PARENTS),
            "tree": SUBJECT_TREE,
        },
        "support": {
            "linux": {"path": "linux-start.S", "sha256": LINUX_START_SHA256, "size": LINUX_START_SIZE},
            "windows": {"path": "windows-chkstk.S", "sha256": WINDOWS_CHKSTK_SHA256, "size": WINDOWS_CHKSTK_SIZE},
        },
        "tool": {"id": TOOL_ID, "path": TOOL_PATH, "sha256": reference("tool")},
        "transport": "temporary-actions-text-only",
        "workflow": {"path": WORKFLOW_PATH, "sha256": reference("workflow")},
    }


def set_public_streams(manifest: dict[str, Any], stdout: bytes, stderr: bytes = b"") -> None:
    public = manifest["platforms"][0]["commands"]["public_run"]
    public["stdout"] = byte_record(stdout)
    public["stderr"] = byte_record(stderr)


def validate_replay_pair(accepted: Any, fresh: Any) -> None:
    require(replay_projection(fresh) == replay_projection(accepted), "fresh manifest differs in a claim-bearing replay field")


def validate_accepted_manifest_unchanged(path: pathlib.Path, before: bytes) -> None:
    require(path.read_bytes() == before, "accepted manifest immutability was violated")


def self_test_schema_unknown() -> None:
    schema = {
        "additionalProperties": False,
        "properties": {"value": {"type": "integer"}},
        "required": ["value"],
        "type": "object",
    }
    validate_against_schema({"value": 1}, schema)
    try:
        validate_against_schema({"value": 1, "unknown": 2}, schema)
    except EvidenceError:
        return
    raise EvidenceError("schema unknown-field mutation unexpectedly passed")


def self_test_schema_float() -> None:
    try:
        parse_json_bytes(b'{"value":1.0}\n', "floating fixture")
    except EvidenceError:
        return
    raise EvidenceError("floating JSON mutation unexpectedly passed")


def self_test_schema_duplicate() -> None:
    try:
        parse_json_bytes(b'{"value":1,"value":2}\n', "duplicate fixture")
    except EvidenceError:
        return
    raise EvidenceError("duplicate JSON mutation unexpectedly passed")


def self_test_canonical_byte_forms() -> None:
    canonical = b'{"value":1}\n'
    require(parse_json_bytes(canonical, "canonical fixture", canonical=True) == {"value": 1}, "canonical JSON fixture drifted")
    for label, payload in {
        "pretty": b'{\n  "value": 1\n}\n',
        "crlf": b'{"value":1}\r\n',
        "bom": b'\xef\xbb\xbf{"value":1}\n',
        "missing-lf": b'{"value":1}',
    }.items():
        expect_evidence_failure(lambda payload=payload: parse_json_bytes(payload, label, canonical=True), f"canonical {label}")


def self_test_tool_banners() -> None:
    for platform_name in PLATFORM_NAMES:
        for tool in TOOL_NAMES:
            record = fixture_tool_record(platform_name, tool)
            validate_tool_record(record, platform_name, tool)
            mutated = clone_json(record)
            raw = decode_byte_record(mutated["version"]["stdout"], "fixture tool stdout")
            if tool == "rustc":
                raw = raw.replace(RUST_COMMIT.encode("ascii"), ("0" * 40).encode("ascii"))
            elif tool == "cargo":
                raw = raw.replace(f"release: {RUST_VERSION}".encode("ascii"), b"release: 0.0.0")
            elif tool == "clang":
                raw = raw.replace(LLVM_VERSION.encode("ascii"), b"0.0.0", 1)
            elif tool == "lld":
                raw = raw.replace(b"LLD ", b"linker ", 1)
            else:
                raw = raw.replace(LLVM_VERSION.encode("ascii"), b"0.0.0", 1)
            mutated["version"]["stdout"] = byte_record(raw)
            expect_evidence_failure(
                lambda mutated=mutated, platform_name=platform_name, tool=tool: validate_tool_record(mutated, platform_name, tool),
                f"{platform_name} {tool} banner",
            )


def fixture_reproduce_bytes() -> bytes:
    return ("# CAP-024 fixture\n\n" + "\n".join(REPRODUCE_REQUIRED) + "\n").encode("utf-8")


def self_test_aggregate_staging(case: str) -> None:
    with tempfile.TemporaryDirectory(prefix="cap024-aggregate-self-test-") as directory:
        bundle = pathlib.Path(directory)
        write_json(bundle / "oracle.json", expected_oracle())
        (bundle / "REPRODUCE.md").write_bytes(fixture_reproduce_bytes())
        claim_index = parse_json_bytes(
            (REPOSITORY_ROOT / "claim-verification/claims.json").read_bytes(),
            "aggregate self-test claim index",
            canonical=False,
        )
        validate_aggregate_staging(bundle, claim_index)
        if case == "aggregate_oracle_drift":
            oracle = expected_oracle()
            oracle["records"][0]["result"][0] = 0
            write_json(bundle / "oracle.json", oracle)
            action = lambda: validate_aggregate_staging(bundle, claim_index)
        elif case == "aggregate_reproduce_drift":
            payload = fixture_reproduce_bytes().replace(
                SUBJECT_TREE.encode("ascii"), b"missing-tree"
            )
            (bundle / "REPRODUCE.md").write_bytes(payload)
            action = lambda: validate_aggregate_staging(bundle, claim_index)
        elif case == "aggregate_claim_index_drift":
            mutated = clone_json(claim_index)
            claim = next(
                record
                for record in mutated["claims"]
                if record.get("id") == CLAIM_ID
            )
            claim["status"] = "candidate"
            action = lambda: validate_aggregate_staging(bundle, mutated)
        elif case == "aggregate_manifest_hash_drift":
            manifest = build_manifest(
                fixture_platform_record("linux-x86_64"),
                fixture_platform_record("windows-x86_64"),
                bundle,
            )
            (bundle / "REPRODUCE.md").write_bytes(
                fixture_reproduce_bytes() + b"post-manifest drift\n"
            )
            action = lambda: validate_manifest_file_hashes(bundle, manifest)
        else:
            raise EvidenceError(f"unknown aggregate staging case {case}")
        expect_evidence_failure(action, case)


def self_test_aggregate_combined_failures() -> None:
    with tempfile.TemporaryDirectory(prefix="cap024-aggregate-combined-") as directory:
        root = pathlib.Path(directory)
        bundle = root / "bundle"
        bundle.mkdir()
        write_json(bundle / "oracle.json", expected_oracle())
        transport = root / "transport"
        linux = transport / "linux-x86_64"
        windows = transport / "windows-x86_64"
        linux.mkdir(parents=True)
        windows.mkdir(parents=True)
        write_json(
            linux / "failure.json",
            {
                "failure": {
                    "message": "fixture capture failed",
                    "mode": "capture",
                    "platform": "linux-x86_64",
                },
                "schema_id": FAILURE_SCHEMA_ID,
            },
        )
        write_json(
            windows / "capture.json",
            {
                "platform": fixture_platform_record("windows-x86_64"),
                "schema_id": "cap024-platform-capture-v1",
                "subject_commit": SUBJECT_COMMIT,
                "tool_id": TOOL_ID,
            },
        )
        keys = (TRANSPORT_ENV, LINUX_JOB_RESULT_ENV, WINDOWS_JOB_RESULT_ENV)
        before = {key: os.environ.get(key) for key in keys}
        try:
            os.environ[TRANSPORT_ENV] = str(transport)
            os.environ[LINUX_JOB_RESULT_ENV] = "failure"
            os.environ[WINDOWS_JOB_RESULT_ENV] = "success"
            args = argparse.Namespace(
                bundle=str(bundle),
                linux=None,
                linux_record=None,
                output=None,
                windows=None,
                windows_record=None,
            )
            try:
                aggregate_records(args)
            except EvidenceError as exc:
                message = str(exc)
                require(
                    "aggregate staging bundle files invalid" in message
                    and "linux-x86_64 capture job result was failure" in message,
                    "aggregate did not report both staging and platform failures",
                )
            else:
                raise EvidenceError("aggregate staging plus platform failure unexpectedly passed")
        finally:
            for key, value in before.items():
                if value is None:
                    os.environ.pop(key, None)
                else:
                    os.environ[key] = value


def run_negative_case(case: str) -> None:
    require(case in NEGATIVE_CASES, f"unknown negative_case {case}")
    if case == "schema_unknown":
        self_test_schema_unknown()
        return
    if case == "schema_float":
        self_test_schema_float()
        return
    if case == "schema_duplicate":
        self_test_schema_duplicate()
        return
    if case == "oracle_drift":
        mutated = expected_oracle()
        mutated["records"][0]["result"][0] = 0
        expect_evidence_failure(lambda: validate_oracle(mutated), case)
        return
    if case.startswith("aggregate_"):
        self_test_aggregate_staging(case)
        return

    manifest = fixture_manifest()
    validate_manifest(manifest)
    if case == "artifact_pair":
        manifest["platforms"][0]["artifacts"]["llvm"]["second"]["sha256"] = sha256_bytes(b"different artifact")
        action = lambda: validate_manifest(manifest)
    elif case == "artifact_path":
        manifest["platforms"][0]["artifacts"]["llvm"]["first"]["path"] += ".wrong"
        action = lambda: validate_manifest(manifest)
    elif case == "artifact_producer":
        manifest["platforms"][0]["artifacts"]["llvm"]["first"]["producer_command"] = "llvm_assemble_first"
        action = lambda: validate_manifest(manifest)
    elif case == "command_argv":
        manifest["platforms"][0]["commands"]["llvm_verify_first"]["argv"].append("--arbitrary")
        action = lambda: validate_manifest(manifest)
    elif case == "command_env":
        manifest["platforms"][0]["commands"]["llvm_verify_first"]["env"]["inheritance"] = "runner-substrate-observation-only"
        action = lambda: validate_manifest(manifest)
    elif case == "public_missing":
        set_public_streams(manifest, b"diagnostic\n")
        action = lambda: validate_manifest(manifest)
    elif case == "public_duplicate":
        set_public_streams(manifest, b"Exit code: 91\nExit code: 91\n")
        action = lambda: validate_manifest(manifest)
    elif case == "public_wrong_exit":
        set_public_streams(manifest, b"Exit code: 90\n")
        action = lambda: validate_manifest(manifest)
    elif case == "public_output":
        set_public_streams(manifest, b"Exit code: 91\nOutput: payload\n")
        action = lambda: validate_manifest(manifest)
    elif case == "public_error":
        set_public_streams(manifest, b"Exit code: 91\nError output: payload\n")
        action = lambda: validate_manifest(manifest)
    elif case == "public_prefixed_exit":
        set_public_streams(manifest, b"prefix Exit code: 91\n")
        action = lambda: validate_manifest(manifest)
    elif case == "public_exit_in_stderr":
        set_public_streams(manifest, b"diagnostic\n", b"Exit code: 91\n")
        action = lambda: validate_manifest(manifest)
    elif case == "included_replay_difference":
        fresh = clone_json(manifest)
        fresh["platforms"][0]["artifacts"]["llvm"]["first"]["sha256"] = sha256_bytes(b"included drift")
        action = lambda: validate_replay_pair(manifest, fresh)
    elif case == "excluded_replay_leaves":
        accepted_bytes = canonical_json_bytes(manifest)
        fresh = clone_json(manifest)
        fresh["platforms"][0]["observations"]["kernel"] = "fresh-kernel-observation"
        validate_manifest(fresh)
        validate_replay_pair(manifest, fresh)
        observations = build_fresh_observations(accepted_bytes, fresh)
        validate_fresh_observations(observations, accepted_bytes, fresh)
        wrong_sha = clone_json(observations)
        wrong_sha["accepted_manifest_sha256"] = "0" * 64
        expect_evidence_failure(lambda: validate_fresh_observations(wrong_sha, accepted_bytes, fresh), "fresh observation SHA")
        wrong_order = clone_json(observations)
        wrong_order["records"][0], wrong_order["records"][1] = wrong_order["records"][1], wrong_order["records"][0]
        expect_evidence_failure(lambda: validate_fresh_observations(wrong_order, accepted_bytes, fresh), "fresh observation order")
        wrong_value = clone_json(observations)
        wrong_value["records"][0]["value"] = "arbitrary-value"
        expect_evidence_failure(lambda: validate_fresh_observations(wrong_value, accepted_bytes, fresh), "fresh observation value")
        return
    elif case == "extra_bundle_file":
        action = lambda: validate_bundle_file_names(REQUIRED_BUNDLE_FILES | {"extra.json"})
    elif case == "reproduce_drift":
        valid = fixture_reproduce_bytes()
        validate_reproduce_bytes(valid)
        action = lambda: validate_reproduce_bytes(valid.replace(SUBJECT_TREE.encode("ascii"), b"missing-tree"))
    elif case == "accepted_manifest_immutable":
        with tempfile.TemporaryDirectory(prefix="cap024-self-test-") as directory:
            path = pathlib.Path(directory) / "manifest.json"
            before = canonical_json_bytes(manifest)
            path.write_bytes(before)
            validate_accepted_manifest_unchanged(path, before)
            path.write_bytes(before + b" ")
            expect_evidence_failure(lambda: validate_accepted_manifest_unchanged(path, before), case)
        return
    else:
        raise EvidenceError(f"negative case {case} has no behavioral fixture")
    expect_evidence_failure(action, case)


def self_test_locked_vendor() -> None:
    """Exercise lock parsing, archive verification, extraction, and Cargo routing."""

    with tempfile.TemporaryDirectory(prefix="aero-cap024-vendor-selftest-") as temporary:
        root = pathlib.Path(temporary)
        cache = root / "cache"
        cache.mkdir()
        package_name = "fixture-dep"
        package_version = "1.2.3"
        package_root = f"{package_name}-{package_version}"
        archive_path = cache / f"{package_root}.crate"
        files = {
            "Cargo.toml": b'[package]\nname = "fixture-dep"\nversion = "1.2.3"\nedition = "2021"\n',
            "src/lib.rs": b"pub fn fixture() -> u32 { 7 }\n",
        }
        with tarfile.open(archive_path, mode="w:gz") as archive:
            for relative, payload in files.items():
                info = tarfile.TarInfo(f"{package_root}/{relative}")
                info.mode = 0o644
                info.mtime = 0
                info.size = len(payload)
                archive.addfile(info, io.BytesIO(payload))
        checksum, _ = file_identity(archive_path)
        lock_path = root / "Cargo.lock"
        lock_path.write_text(
            "# This file is automatically @generated by Cargo.\n"
            "version = 4\n\n"
            "[[package]]\n"
            'name = "compiler"\n'
            'version = "0.3.0"\n\n'
            "[[package]]\n"
            f'name = "{package_name}"\n'
            f'version = "{package_version}"\n'
            f'source = "{CRATES_IO_SOURCE}"\n'
            f'checksum = "{checksum}"\n',
            encoding="utf-8",
            newline="\n",
        )
        platform_root = root / "platform"
        materialize_locked_vendor(
            lock_path, platform_root, cache, expected_registry_count=1
        )
        config = (platform_root / "cargo-home/config.toml").read_text(
            encoding="utf-8"
        )
        require(
            'replace-with = "vendored-sources"' in config
            and (platform_root / f"cargo-vendor/{package_root}/Cargo.toml").is_file(),
            LOCKFILE_VENDOR_BOUNDARY,
        )
        checksum_record = read_json(
            platform_root / f"cargo-vendor/{package_root}/.cargo-checksum.json",
            "self-test vendor checksum",
        )
        require(
            checksum_record == {
                "files": {
                    relative: sha256_bytes(payload)
                    for relative, payload in files.items()
                },
                "package": checksum,
            },
            "self-test vendor checksum metadata drifted",
        )
        unauthorized_lock = root / "unauthorized.lock"
        unauthorized_lock.write_text(
            lock_path.read_text(encoding="utf-8").replace(
                CRATES_IO_SOURCE, "registry+https://example.invalid/index"
            ),
            encoding="utf-8",
            newline="\n",
        )
        expect_evidence_failure(
            lambda: locked_registry_packages(
                unauthorized_lock, expected_registry_count=1
            ),
            "locked vendor alternate source",
        )
        expect_evidence_failure(
            lambda: acquire_locked_crate(
                {
                    "checksum": "0" * 64,
                    "name": package_name,
                    "version": package_version,
                },
                cache,
            ),
            "locked vendor checksum drift",
        )
        unsafe_archive = root / "unsafe.crate"
        with tarfile.open(unsafe_archive, mode="w:gz") as archive:
            payload = b"escape"
            info = tarfile.TarInfo(f"{package_root}/..\\escape")
            info.size = len(payload)
            archive.addfile(info, io.BytesIO(payload))
        unsafe_vendor = root / "unsafe-vendor"
        unsafe_vendor.mkdir()
        expect_evidence_failure(
            lambda: extract_locked_crate(
                {
                    "checksum": checksum,
                    "name": package_name,
                    "version": package_version,
                },
                unsafe_archive,
                unsafe_vendor,
            ),
            "locked vendor portable path escape",
        )


def self_test_exact_rust_selection() -> None:
    """Prove a different runner default cannot outrank the exact release."""

    with tempfile.TemporaryDirectory(prefix="aero-cap024-rust-selftest-") as temporary:
        manager_home = pathlib.Path(temporary) / ("rust" + "up")
        toolchains = manager_home / "toolchains"
        suffix = ".exe" if os.name == "nt" else ""
        for name in (f"{RUST_VERSION}-fixture-host", "stable-fixture-host"):
            binary = toolchains / name / "bin"
            binary.mkdir(parents=True)
            (binary / f"cargo{suffix}").write_bytes(b"fixture cargo")
            (binary / f"rustc{suffix}").write_bytes(b"fixture rustc")
        manager_home.mkdir(exist_ok=True)
        (manager_home / "settings.toml").write_text(
            'default_toolchain = "stable-fixture-host"\n',
            encoding="utf-8",
            newline="\n",
        )
        manager_home_key = "RUST" + "UP_HOME"
        selected_key = "RUST" + "UP_TOOLCHAIN"
        keys = ("CAP024_RUST_ROOT", manager_home_key, selected_key)
        before = {key: os.environ.get(key) for key in keys}
        try:
            os.environ.pop("CAP024_RUST_ROOT", None)
            os.environ[manager_home_key] = str(manager_home)
            os.environ[selected_key] = "stable-fixture-host"
            selected = discover_rust_root()
            require(
                selected == (toolchains / f"{RUST_VERSION}-fixture-host").resolve(),
                "exact Rust release did not outrank the runner default",
            )
        finally:
            for key, value in before.items():
                if value is None:
                    os.environ.pop(key, None)
                else:
                    os.environ[key] = value


def run_self_test(negative_case: str | None = None) -> None:
    verify_embedded_support()
    oracle = expected_oracle()
    validate_oracle(oracle)
    require(oracle["records"][0]["result"] == [1, 122, 167, 135, 181, 4940, 5573, 1], "ordinary oracle drifted")
    require(oracle["records"][1]["result"] == [1, -24, 18, 2147483623, 0, -37, 2147483641, 1], "wrapping oracle drifted")
    require(oracle["records"][2]["result"] == [1, -3, 0, 0, 0, 5, 4, 0], "activation oracle drifted")
    require(oracle["records"][3]["result"] == [1, 1, 2, 1, 2, 3, 3, 0], "tie oracle drifted")
    require(all(record["result"] == [0] * 8 for record in oracle["records"][4:]), "malformed-header oracle drifted")
    require(all(record["source"] == record["source_after_call"] and record["source_preserved"] for record in oracle["records"]), "source preservation oracle drifted")
    locked_packages = locked_registry_packages(REPOSITORY_ROOT / "src/compiler/Cargo.lock")
    require(len(locked_packages) == 112 and all(valid_sha256(package["checksum"]) for package in locked_packages), LOCKFILE_VENDOR_BOUNDARY)
    self_test_locked_vendor()
    self_test_exact_rust_selection()
    self_test_aggregate_combined_failures()
    record = byte_record(b"canonical bytes\n")
    require(decode_byte_record(record, "self-test byte record") == b"canonical bytes\n", "byte record roundtrip drifted")
    require(
        TARGET_BYTE_ENVIRONMENT_ANCHOR
        in json.dumps(command_environment("linux-x86_64", "llvm_verify_first"), sort_keys=True),
        "target-byte command environment must encode no ambient inheritance",
    )
    self_test_canonical_byte_forms()
    self_test_tool_banners()
    if negative_case is None:
        for case in sorted(NEGATIVE_CASES):
            run_negative_case(case)
    else:
        run_negative_case(negative_case)


def replay_bundle(args: argparse.Namespace) -> None:
    bundle = pathlib.Path(args.bundle).resolve()
    accepted_path = bundle / "manifest.json"
    before = accepted_path.read_bytes()
    accepted, accepted_bytes = validate_bundle(bundle)
    require(before == accepted_bytes, "accepted manifest bytes changed during validation")
    if args.fresh_manifest:
        fresh = read_json(pathlib.Path(args.fresh_manifest), "fresh replay manifest")
        schema = read_json(REPOSITORY_ROOT / SCHEMA_PATH, "CAP-024 schema")
        validate_manifest(fresh, schema)
        validate_replay_pair(accepted, fresh)
        generated = build_fresh_observations(accepted_bytes, fresh)
        if args.fresh_observations:
            observations = read_json(pathlib.Path(args.fresh_observations), "fresh observations")
            validate_fresh_observations(observations, accepted_bytes, fresh)
            require(observations == generated, "fresh observations differ from the exact closed projection leaves")
        elif args.emit_fresh_observations:
            write_json(pathlib.Path(args.emit_fresh_observations).resolve(), generated)
        else:
            raise EvidenceError("replay with --fresh-manifest requires --fresh-observations or --emit-fresh-observations")
    else:
        require(not args.fresh_observations and not args.emit_fresh_observations, "fresh observation options require --fresh-manifest")
        require(args.verify_only, "replay without a fresh manifest requires --verify-only")
    validate_accepted_manifest_unchanged(accepted_path, before)


def validate_mode(args: argparse.Namespace) -> None:
    validate_bundle(pathlib.Path(args.bundle).resolve())


def write_failure_record(path: pathlib.Path, mode: str, platform_name: str | None, message: str) -> None:
    value = {
        "failure": {"message": message, "mode": mode, "platform": platform_name},
        "schema_id": FAILURE_SCHEMA_ID,
    }
    write_json(path, value)


def parser() -> argparse.ArgumentParser:
    result = ClosedArgumentParser(description="CAP-024 accepted-head evidence capture and replay")
    result.add_argument("--mode", required=True, choices=["capture", "aggregate", "validate", "replay", "self-test"])
    result.add_argument("--platform", choices=PLATFORM_NAMES)
    result.add_argument("--subject-dir", "--subject", dest="subject_dir")
    result.add_argument("--work-dir", "--work", dest="work_dir")
    result.add_argument("--llvm-dir", "--llvm", dest="llvm_dir")
    result.add_argument("--rust-dir", "--rust", dest="rust_dir")
    result.add_argument("--llvm-archive")
    result.add_argument("--acquire-directory")
    result.add_argument("--output")
    result.add_argument("--failure-output")
    result.add_argument("--bundle")
    result.add_argument("--linux-record")
    result.add_argument("--windows-record")
    result.add_argument("--linux")
    result.add_argument("--windows")
    result.add_argument("--fresh-manifest")
    result.add_argument("--fresh-observations")
    result.add_argument("--emit-fresh-observations")
    result.add_argument("--verify-only", action="store_true")
    result.add_argument("--negative-case", choices=sorted(NEGATIVE_CASES))
    return result


def require_mode_arguments(args: argparse.Namespace) -> None:
    if args.mode == "capture":
        require(args.platform is not None, "capture requires --platform")
    elif args.mode == "aggregate":
        pass
    elif args.mode in {"validate", "replay"}:
        require(args.bundle is not None, f"{args.mode} requires --bundle")


def emit_success(mode: str, negative_case: str | None = None) -> None:
    if negative_case is not None:
        sys.stdout.buffer.write(canonical_json_bytes({"case": negative_case, "mode": mode, "ok": True}))
        return
    records = {
        "self-test": SELF_TEST_SUCCESS,
        "validate": VALIDATE_SUCCESS,
        "replay": REPLAY_SUCCESS,
        "capture": CAPTURE_SUCCESS,
        "aggregate": AGGREGATE_SUCCESS,
    }
    sys.stdout.buffer.write(records[mode].encode("utf-8") + b"\n")


def main(argv: Sequence[str] | None = None) -> int:
    args: argparse.Namespace | None = None
    try:
        args = parser().parse_args(argv)
        require_mode_arguments(args)
        if args.mode == "self-test":
            run_self_test(args.negative_case)
        elif args.mode == "validate":
            validate_mode(args)
        elif args.mode == "replay":
            replay_bundle(args)
        elif args.mode == "capture":
            capture = capture_platform(args)
            if args.output:
                output = pathlib.Path(args.output).resolve()
            else:
                transport = pathlib.Path(os.environ.get(TRANSPORT_ENV, REPOSITORY_ROOT / "cap024-evidence-transport")).resolve()
                output = transport / args.platform / "capture.json"
            write_json(output, capture)
        elif args.mode == "aggregate":
            aggregate_records(args)
        emit_success(args.mode, args.negative_case)
        return 0
    except Exception as exc:
        mode = args.mode if args is not None else "invocation"
        platform_name = args.platform if args is not None else None
        failure_output = args.failure_output if args is not None else None
        failure_path = pathlib.Path(failure_output).resolve() if failure_output else None
        if failure_path is None and mode in {"capture", "aggregate"}:
            transport = pathlib.Path(os.environ.get(TRANSPORT_ENV, REPOSITORY_ROOT / "cap024-evidence-transport")).resolve()
            failure_path = transport / (platform_name if mode == "capture" else "aggregate") / "failure.json"
        if failure_path is not None:
            try:
                write_failure_record(failure_path, mode, platform_name, str(exc))
            except OSError:
                pass
        sys.stderr.buffer.write(canonical_json_bytes({"error": str(exc), "mode": mode, "ok": False}))
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
