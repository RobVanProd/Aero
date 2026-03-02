#!/usr/bin/env python3
"""
GGUF inference runner via llama-cpp-python.

Prints parseable metrics for gguf_compare.py:
- tokens/sec
- prompt_eval_ms
- peak_vram_gb (0.0 for CPU path)
"""

from __future__ import annotations

import argparse
import time
from typing import Any, Dict


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run GGUF inference with llama-cpp-python.")
    parser.add_argument("--model", required=True)
    parser.add_argument("--prompt", required=True)
    parser.add_argument("--max-tokens", type=int, default=128)
    parser.add_argument("--n-ctx", type=int, default=2048)
    parser.add_argument("--threads", type=int, default=0, help="0 => auto")
    parser.add_argument("--n-gpu-layers", type=int, default=0)
    parser.add_argument("--temperature", type=float, default=0.7)
    parser.add_argument("--top-p", type=float, default=0.95)
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--echo", action="store_true", help="Print generated text.")
    return parser.parse_args()


def main() -> int:
    args = parse_args()

    try:
        from llama_cpp import Llama  # type: ignore
    except Exception as exc:  # pragma: no cover
        print(f"Failed to import llama_cpp: {exc}")
        print("Install with: python -m pip install llama-cpp-python")
        return 2

    llama_kwargs: Dict[str, Any] = {
        "model_path": args.model,
        "n_ctx": args.n_ctx,
        "n_gpu_layers": args.n_gpu_layers,
        "verbose": False,
        "seed": args.seed,
    }
    if args.threads > 0:
        llama_kwargs["n_threads"] = args.threads

    model = Llama(**llama_kwargs)

    start = time.perf_counter()
    output = model.create_completion(
        prompt=args.prompt,
        max_tokens=args.max_tokens,
        temperature=args.temperature,
        top_p=args.top_p,
        stop=[],
    )
    end = time.perf_counter()
    elapsed_s = max(1e-9, end - start)

    usage = output.get("usage", {}) if isinstance(output, dict) else {}
    completion_tokens = int(usage.get("completion_tokens", 0))

    timings = output.get("timings", {}) if isinstance(output, dict) else {}
    prompt_eval_ms = float(timings.get("prompt_ms", 0.0))
    predicted_ms = float(timings.get("predicted_ms", 0.0))

    tokens_per_second = 0.0
    if completion_tokens > 0 and predicted_ms > 0:
        tokens_per_second = completion_tokens / (predicted_ms / 1000.0)
    elif completion_tokens > 0:
        tokens_per_second = completion_tokens / elapsed_s

    print(f"tokens/sec: {tokens_per_second:.6f}")
    print(f"prompt_eval_ms: {prompt_eval_ms:.6f}")
    # CPU runner does not expose GPU memory telemetry.
    print("peak_vram_gb: 0.000000")
    print(f"completion_tokens: {completion_tokens}")
    print(f"wall_seconds: {elapsed_s:.6f}")

    if args.echo:
        text = ""
        if isinstance(output, dict):
            choices = output.get("choices", [])
            if choices:
                text = str(choices[0].get("text", ""))
        print("output_text_begin")
        print(text)
        print("output_text_end")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())

