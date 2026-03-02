# GGUF Cross-Framework Benchmarks

This folder contains a reproducible benchmark harness for comparing GGUF
inference performance across:

- Aero
- llama.cpp
- PyTorch reference path

The harness is command-template driven so you can benchmark any backend that can
print parseable metrics.

## Files

- `gguf_compare.py` - main benchmark runner (20-run aggregation + reports)
- `config.rx7800xt.example.json` - template config for RX 7800 XT / `gfx1101`
- `config.mock.json` - local validation config that uses synthetic output
- `config.local_llama_cpp_python.json` - immediate real-model local baseline
- `mock_backend.py` - synthetic backend output generator for testing
- `pytorch_reference_runner.py` - optional PyTorch baseline runner
- `llama_cpp_runner.py` - GGUF runner via `llama-cpp-python`

## Quick Start

### 1) Validate harness logic locally (no model required)

```bash
python benchmarks/gguf/gguf_compare.py --config benchmarks/gguf/config.mock.json
```

Output artifacts are written to `benchmarks/results/gguf/`:

- `<name>_<timestamp>.json`
- `<name>_<timestamp>.md`
- `<name>_<timestamp>.html`

### 2) Run real GGUF comparison on RX 7800 XT

1. Put your model in `models/` (example):
   - `models/Meta-Llama-3.1-8B-Instruct-Q5_K_M.gguf`
2. Copy and edit config:
   - `benchmarks/gguf/config.rx7800xt.example.json`
3. Run:

```bash
python benchmarks/gguf/gguf_compare.py --config benchmarks/gguf/config.rx7800xt.example.json
```

### 3) Run immediate local baseline with installed `llama-cpp-python`

```bash
python benchmarks/gguf/gguf_compare.py --config benchmarks/gguf/config.local_llama_cpp_python.json
```

## Config Notes

- `metric.direction`:
  - `"higher"` for throughput metrics like tokens/sec
  - `"lower"` for latency metrics like seconds/run
- `strict`:
  - `true`: fail immediately on first bad run
  - `false`: continue and report partial results
- `metric_regexes`: regex list where capture group 1 is numeric metric value
- `aux_metrics`: optional extra parsed metrics (prompt eval ms, peak VRAM, etc.)

## Expected Output Schema

Each run includes:

- process exit code
- wall-clock seconds
- primary metric
- optional parsed aux metrics

Each backend includes summary stats:

- mean / median / min / max / p95 / stdev
- success run count
- failed run count
