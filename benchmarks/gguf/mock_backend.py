#!/usr/bin/env python3
"""
Mock GGUF backend output generator for harness validation.
"""

from __future__ import annotations

import argparse
import random
import time


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Emit fake backend metrics.")
    parser.add_argument("--backend", required=True)
    parser.add_argument("--min-tps", type=float, required=True)
    parser.add_argument("--max-tps", type=float, required=True)
    parser.add_argument("--prompt-ms", type=float, default=40.0)
    parser.add_argument("--vram-gb", type=float, default=7.5)
    parser.add_argument("--sleep-ms", type=float, default=10.0)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    random.seed(time.time_ns())

    tps = random.uniform(args.min_tps, args.max_tps)
    prompt_ms = args.prompt_ms * random.uniform(0.96, 1.04)
    vram = args.vram_gb * random.uniform(0.99, 1.01)

    time.sleep(max(0.0, args.sleep_ms) / 1000.0)

    print(f"backend={args.backend}")
    print(f"tokens/sec: {tps:.4f}")
    print(f"prompt_eval_ms: {prompt_ms:.4f}")
    print(f"peak_vram_gb: {vram:.4f}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

