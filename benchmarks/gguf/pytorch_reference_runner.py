#!/usr/bin/env python3
"""
PyTorch reference runner for comparison harness.

Notes:
- PyTorch does not natively load GGUF files. If `--model` points to `.gguf`,
  this script exits with code 2 and a clear message.
- For Hugging Face model IDs / local Transformer checkpoints, it can run a
  minimal generate() timing path when `torch` + `transformers` are installed.
"""

from __future__ import annotations

import argparse
import sys
import time
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="PyTorch baseline generation runner.")
    parser.add_argument("--model", required=True, help="Model path or HF model id.")
    parser.add_argument("--prompt", required=True)
    parser.add_argument("--max-tokens", type=int, default=256)
    parser.add_argument("--device", default="cuda")
    parser.add_argument(
        "--force-tokens-per-second",
        type=float,
        default=-1.0,
        help="Emit a fixed synthetic value and exit successfully (debug/testing only).",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()

    if args.force_tokens_per_second > 0:
        tps = args.force_tokens_per_second
        print(f"tokens/sec: {tps:.4f}")
        print("prompt_eval_ms: 0.0000")
        print("peak_vram_gb: 0.0000")
        return 0

    model_str = args.model.strip()
    if model_str.lower().endswith(".gguf") or Path(model_str).suffix.lower() == ".gguf":
        print(
            "PyTorch reference runner cannot load GGUF directly. "
            "Use a non-GGUF Transformer checkpoint/model id for this backend."
        )
        return 2

    try:
        import torch  # type: ignore
        from transformers import AutoModelForCausalLM, AutoTokenizer  # type: ignore
    except Exception as exc:  # pragma: no cover - environment dependent
        print(f"Missing dependencies for PyTorch runner: {exc}")
        print("Install with: pip install torch transformers")
        return 2

    device = args.device
    if device.startswith("cuda") and not torch.cuda.is_available():
        device = "cpu"

    tokenizer = AutoTokenizer.from_pretrained(model_str)
    model = AutoModelForCausalLM.from_pretrained(model_str)
    model.eval()
    if device != "cpu":
        model.to(device)

    inputs = tokenizer(args.prompt, return_tensors="pt")
    if device != "cpu":
        inputs = {k: v.to(device) for k, v in inputs.items()}

    prompt_start = time.perf_counter()
    with torch.no_grad():
        _ = model(**inputs)
    prompt_end = time.perf_counter()
    prompt_eval_ms = (prompt_end - prompt_start) * 1000.0

    gen_start = time.perf_counter()
    with torch.no_grad():
        out = model.generate(
            **inputs,
            max_new_tokens=args.max_tokens,
            do_sample=False,
        )
    gen_end = time.perf_counter()

    prompt_len = int(inputs["input_ids"].shape[-1])
    total_len = int(out.shape[-1])
    generated_tokens = max(0, total_len - prompt_len)
    elapsed = max(1e-9, gen_end - gen_start)
    tokens_per_second = generated_tokens / elapsed if generated_tokens > 0 else 0.0

    peak_vram_gb = 0.0
    if device != "cpu" and torch.cuda.is_available():
        peak_vram_bytes = float(torch.cuda.max_memory_allocated())
        peak_vram_gb = peak_vram_bytes / (1024.0 ** 3)

    print(f"tokens/sec: {tokens_per_second:.4f}")
    print(f"prompt_eval_ms: {prompt_eval_ms:.4f}")
    print(f"peak_vram_gb: {peak_vram_gb:.4f}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

