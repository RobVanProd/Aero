#!/usr/bin/env python3
"""
Cross-framework GGUF benchmark harness.

This script runs comparable command-line inference workloads (Aero, llama.cpp,
PyTorch, or any custom backend), aggregates N-run metrics, and writes:
1) machine-readable JSON results
2) Markdown summary report
3) HTML report with a simple inline SVG chart

It is intentionally framework-agnostic: each backend is configured with a
command template plus regexes for metric extraction.
"""

from __future__ import annotations

import argparse
import html
import json
import os
import re
import statistics
import subprocess
import sys
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Sequence, Tuple


REPO_ROOT = Path(__file__).resolve().parents[2]


@dataclass
class MetricConfig:
    key: str
    unit: str
    direction: str  # "higher" or "lower"

    @classmethod
    def from_config(cls, raw: Dict[str, Any]) -> "MetricConfig":
        metric = raw.get("metric", {})
        key = str(metric.get("key", "tokens_per_second"))
        unit = str(metric.get("unit", "tokens/s"))
        direction = str(metric.get("direction", "higher")).lower()
        if direction not in {"higher", "lower"}:
            raise ValueError("metric.direction must be 'higher' or 'lower'")
        return cls(key=key, unit=unit, direction=direction)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run GGUF backend comparisons.")
    parser.add_argument(
        "--config",
        required=True,
        help="Path to benchmark config JSON",
    )
    parser.add_argument(
        "--output-json",
        default="",
        help="Override output JSON file path",
    )
    parser.add_argument(
        "--output-md",
        default="",
        help="Override output Markdown file path",
    )
    parser.add_argument(
        "--output-html",
        default="",
        help="Override output HTML file path",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Render commands and exit without running benchmarks",
    )
    parser.add_argument(
        "--runs",
        type=int,
        default=-1,
        help="Override measured run count from config",
    )
    parser.add_argument(
        "--warmup-runs",
        type=int,
        default=-1,
        help="Override warmup run count from config",
    )
    parser.add_argument(
        "--backend",
        default="",
        help=(
            "Run only matching backends (comma-separated). "
            "Matching supports exact names or substring tokens, e.g. 'rocm' or 'aero_rocm,llama_cpp_rocm'."
        ),
    )
    return parser.parse_args()


def load_config(path: Path) -> Dict[str, Any]:
    with path.open("r", encoding="utf-8") as f:
        return json.load(f)


def percentile(values: Sequence[float], p: float) -> float:
    if not values:
        raise ValueError("percentile() requires at least one value")
    if p <= 0:
        return min(values)
    if p >= 100:
        return max(values)
    sorted_values = sorted(values)
    k = (len(sorted_values) - 1) * (p / 100.0)
    lower = int(k)
    upper = min(lower + 1, len(sorted_values) - 1)
    if lower == upper:
        return sorted_values[lower]
    fraction = k - lower
    return sorted_values[lower] * (1.0 - fraction) + sorted_values[upper] * fraction


def summarize(values: Sequence[float]) -> Dict[str, float]:
    if not values:
        return {}
    data = list(values)
    return {
        "mean": float(statistics.mean(data)),
        "median": float(statistics.median(data)),
        "min": float(min(data)),
        "max": float(max(data)),
        "p95": float(percentile(data, 95.0)),
        "stdev": float(statistics.pstdev(data)) if len(data) > 1 else 0.0,
    }


def to_posix(path: Path) -> str:
    return path.resolve().as_posix()


def build_template_variables(config: Dict[str, Any]) -> Dict[str, str]:
    variables: Dict[str, str] = {}
    for k, v in config.get("variables", {}).items():
        variables[str(k)] = str(v)

    model = config.get("model", {})
    prompt = str(config.get("prompt", ""))
    max_tokens = int(config.get("max_tokens", 256))

    model_path = str(model.get("path", ""))
    if model_path:
        model_path = to_posix((REPO_ROOT / model_path) if not Path(model_path).is_absolute() else Path(model_path))

    variables.update(
        {
            "repo_root": to_posix(REPO_ROOT),
            "model_path": model_path,
            "model_name": str(model.get("name", Path(model_path).name if model_path else "")),
            "prompt": prompt,
            "max_tokens": str(max_tokens),
        }
    )
    return variables


def format_command(template: str, variables: Dict[str, str]) -> str:
    try:
        return template.format(**variables)
    except KeyError as exc:
        raise ValueError(f"missing template variable '{exc.args[0]}' in command: {template}") from exc


def parse_backend_filters(raw: str) -> List[str]:
    return [item.strip().lower() for item in raw.split(",") if item.strip()]


def backend_is_selected(name: str, filters: Sequence[str]) -> bool:
    if not filters:
        return True
    lower_name = name.lower()
    return any(token == lower_name or token in lower_name for token in filters)


def parse_numeric_metric(text: str, regexes: Iterable[str]) -> Optional[float]:
    for pattern in regexes:
        match = re.search(pattern, text, flags=re.IGNORECASE | re.MULTILINE)
        if match:
            try:
                return float(match.group(1))
            except ValueError:
                continue
    return None


def normalize_regexes(raw: Any) -> List[str]:
    if raw is None:
        return []
    if isinstance(raw, str):
        return [raw]
    if isinstance(raw, list):
        return [str(x) for x in raw]
    raise ValueError("metric regex must be string or list of strings")


def run_once(
    *,
    backend_name: str,
    command: str,
    cwd: Path,
    timeout_s: int,
    env: Dict[str, str],
    metric_regexes: List[str],
    aux_regexes: Dict[str, List[str]],
    metric_fallback_to_wall_time: bool,
) -> Dict[str, Any]:
    start = time.perf_counter()
    proc = subprocess.run(
        command,
        cwd=str(cwd),
        env=env,
        shell=True,
        capture_output=True,
        text=True,
        timeout=timeout_s,
    )
    end = time.perf_counter()
    wall_s = end - start
    combined = (proc.stdout or "") + "\n" + (proc.stderr or "")

    primary_metric = parse_numeric_metric(combined, metric_regexes)
    if primary_metric is None and metric_fallback_to_wall_time:
        primary_metric = wall_s

    aux_metrics: Dict[str, float] = {}
    for key, regexes in aux_regexes.items():
        value = parse_numeric_metric(combined, regexes)
        if value is not None:
            aux_metrics[key] = value

    return {
        "backend": backend_name,
        "command": command,
        "exit_code": int(proc.returncode),
        "wall_seconds": wall_s,
        "primary_metric": primary_metric,
        "aux_metrics": aux_metrics,
        "stdout": proc.stdout,
        "stderr": proc.stderr,
    }


def output_stem(config: Dict[str, Any]) -> str:
    name = str(config.get("benchmark_name", "gguf_compare")).strip() or "gguf_compare"
    safe = "".join(ch if ch.isalnum() or ch in {"_", "-"} else "_" for ch in name)
    return safe


def format_float(value: Optional[float], digits: int = 4) -> str:
    if value is None:
        return "n/a"
    return f"{value:.{digits}f}"


def ranking_key(direction: str, value: float) -> float:
    return -value if direction == "higher" else value


def generate_markdown_report(result: Dict[str, Any], metric: MetricConfig) -> str:
    bench = result["benchmark"]
    rows = result["results"]

    lines: List[str] = []
    lines.append(f"# GGUF Benchmark Report: {bench['benchmark_name']}")
    lines.append("")
    lines.append(f"- Timestamp (UTC): `{bench['timestamp_utc']}`")
    lines.append(f"- Hardware: `{bench['hardware']}`")
    lines.append(f"- Model: `{bench['model_name']}`")
    lines.append(f"- Model path: `{bench['model_path']}`")
    lines.append(f"- Runs: `{bench['runs']}` measured + `{bench['warmup_runs']}` warmup")
    lines.append(f"- Metric: `{metric.key}` ({metric.unit}, {metric.direction} is better)")
    lines.append("")

    lines.append("## Summary")
    lines.append("")
    lines.append("| Backend | Status | Median metric | Mean metric | Median wall (s) | Success runs | Failed runs |")
    lines.append("|---|---:|---:|---:|---:|---:|---:|")
    for entry in rows:
        summary = entry.get("summary", {})
        metric_summary = summary.get("primary_metric", {})
        wall_summary = summary.get("wall_seconds", {})
        lines.append(
            "| {name} | {status} | {med} | {mean} | {wall} | {ok} | {fail} |".format(
                name=entry["name"],
                status=entry["status"],
                med=format_float(metric_summary.get("median"), 4),
                mean=format_float(metric_summary.get("mean"), 4),
                wall=format_float(wall_summary.get("median"), 4),
                ok=summary.get("success_runs", 0),
                fail=summary.get("failed_runs", 0),
            )
        )

    lines.append("")
    lines.append("## Ranking")
    lines.append("")
    ranked = [r for r in rows if r.get("status") == "ok" and r.get("summary", {}).get("primary_metric", {}).get("median") is not None]
    ranked.sort(key=lambda r: ranking_key(metric.direction, float(r["summary"]["primary_metric"]["median"])))
    for idx, entry in enumerate(ranked, start=1):
        med = entry["summary"]["primary_metric"]["median"]
        lines.append(f"{idx}. `{entry['name']}`: `{med:.4f} {metric.unit}`")

    if not ranked:
        lines.append("- No successful backend metrics.")

    lines.append("")
    lines.append("## Notes")
    lines.append("")
    lines.append("- Report generated by `benchmarks/gguf/gguf_compare.py`.")
    lines.append("- `median` is recommended for headline comparisons.")
    return "\n".join(lines) + "\n"


def build_svg_chart(rows: List[Tuple[str, float]], metric: MetricConfig) -> str:
    if not rows:
        return "<p>No successful benchmark runs to chart.</p>"

    max_label_len = max(len(name) for name, _ in rows)
    left_pad = max(160, max_label_len * 8 + 20)
    bar_area = 560
    row_h = 36
    top_pad = 30
    bottom_pad = 30
    width = left_pad + bar_area + 140
    height = top_pad + bottom_pad + row_h * len(rows)

    max_value = max(v for _, v in rows)
    if max_value <= 0:
        max_value = 1.0

    svg_lines = [
        f'<svg width="{width}" height="{height}" viewBox="0 0 {width} {height}" xmlns="http://www.w3.org/2000/svg">',
        '<rect width="100%" height="100%" fill="#ffffff"/>',
        f'<text x="{left_pad}" y="20" font-family="Segoe UI, sans-serif" font-size="14" fill="#111827">Median {html.escape(metric.key)} ({html.escape(metric.unit)})</text>',
    ]

    for idx, (name, value) in enumerate(rows):
        y = top_pad + idx * row_h
        bar_w = (value / max_value) * bar_area
        label = html.escape(name)
        value_text = f"{value:.4f}"
        svg_lines.append(
            f'<text x="{left_pad - 10}" y="{y + 20}" text-anchor="end" font-family="Segoe UI, sans-serif" font-size="12" fill="#374151">{label}</text>'
        )
        svg_lines.append(
            f'<rect x="{left_pad}" y="{y + 6}" width="{bar_w:.2f}" height="18" fill="#2563eb" rx="3" ry="3"/>'
        )
        svg_lines.append(
            f'<text x="{left_pad + bar_w + 8:.2f}" y="{y + 20}" font-family="Segoe UI, sans-serif" font-size="12" fill="#111827">{value_text}</text>'
        )

    svg_lines.append("</svg>")
    return "\n".join(svg_lines)


def generate_html_report(result: Dict[str, Any], metric: MetricConfig) -> str:
    bench = result["benchmark"]
    rows = result["results"]

    chart_rows: List[Tuple[str, float]] = []
    for entry in rows:
        if entry.get("status") != "ok":
            continue
        med = entry.get("summary", {}).get("primary_metric", {}).get("median")
        if med is None:
            continue
        chart_rows.append((entry["name"], float(med)))

    chart_rows.sort(key=lambda item: ranking_key(metric.direction, item[1]))
    chart_svg = build_svg_chart(chart_rows, metric)

    table_rows = []
    for entry in rows:
        summary = entry.get("summary", {})
        metric_summary = summary.get("primary_metric", {})
        wall_summary = summary.get("wall_seconds", {})
        table_rows.append(
            "<tr>"
            f"<td>{html.escape(entry['name'])}</td>"
            f"<td>{html.escape(entry['status'])}</td>"
            f"<td>{format_float(metric_summary.get('median'), 4)}</td>"
            f"<td>{format_float(metric_summary.get('mean'), 4)}</td>"
            f"<td>{format_float(wall_summary.get('median'), 4)}</td>"
            f"<td>{summary.get('success_runs', 0)}</td>"
            f"<td>{summary.get('failed_runs', 0)}</td>"
            "</tr>"
        )

    return f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>GGUF Benchmark Report - {html.escape(bench['benchmark_name'])}</title>
  <style>
    body {{
      font-family: "Segoe UI", Tahoma, sans-serif;
      margin: 24px auto;
      max-width: 1080px;
      color: #111827;
      line-height: 1.5;
      padding: 0 12px;
    }}
    h1, h2 {{ margin-bottom: 8px; }}
    .meta {{ background: #f3f4f6; padding: 12px 14px; border-radius: 8px; }}
    table {{ border-collapse: collapse; width: 100%; margin-top: 12px; }}
    th, td {{ border: 1px solid #e5e7eb; padding: 8px 10px; text-align: left; }}
    th {{ background: #f9fafb; }}
    .chart {{ margin-top: 20px; border: 1px solid #e5e7eb; border-radius: 8px; padding: 12px; }}
    .small {{ color: #4b5563; font-size: 0.92rem; }}
  </style>
</head>
<body>
  <h1>GGUF Benchmark Report: {html.escape(bench['benchmark_name'])}</h1>
  <div class="meta">
    <div><strong>Timestamp (UTC):</strong> {html.escape(bench['timestamp_utc'])}</div>
    <div><strong>Hardware:</strong> {html.escape(bench['hardware'])}</div>
    <div><strong>Model:</strong> {html.escape(bench['model_name'])}</div>
    <div><strong>Model path:</strong> {html.escape(bench['model_path'])}</div>
    <div><strong>Runs:</strong> {bench['runs']} measured + {bench['warmup_runs']} warmup</div>
    <div><strong>Metric:</strong> {html.escape(metric.key)} ({html.escape(metric.unit)}, {html.escape(metric.direction)} is better)</div>
  </div>

  <h2>Summary</h2>
  <table>
    <thead>
      <tr>
        <th>Backend</th>
        <th>Status</th>
        <th>Median metric</th>
        <th>Mean metric</th>
        <th>Median wall (s)</th>
        <th>Success runs</th>
        <th>Failed runs</th>
      </tr>
    </thead>
    <tbody>
      {''.join(table_rows)}
    </tbody>
  </table>

  <div class="chart">
    <h2>Median Metric Chart</h2>
    {chart_svg}
  </div>

  <p class="small">Generated by benchmarks/gguf/gguf_compare.py</p>
</body>
</html>
"""


def main() -> int:
    args = parse_args()
    config_path = (REPO_ROOT / args.config) if not Path(args.config).is_absolute() else Path(args.config)
    if not config_path.exists():
        print(f"Config not found: {config_path}", file=sys.stderr)
        return 1

    config = load_config(config_path)
    metric = MetricConfig.from_config(config)

    runs = int(config.get("runs", 20)) if args.runs < 0 else args.runs
    warmup_runs = int(config.get("warmup_runs", 3)) if args.warmup_runs < 0 else args.warmup_runs
    timeout_s = int(config.get("timeout_seconds", 900))
    strict = bool(config.get("strict", True))
    metric_fallback_to_wall_time = bool(config.get("metric_fallback_to_wall_time", metric.key == "wall_seconds"))

    variables = build_template_variables(config)
    backends = config.get("backends", [])
    if not backends:
        print("Config must include a non-empty 'backends' list.", file=sys.stderr)
        return 1
    backend_filters = parse_backend_filters(args.backend)

    print(f"Benchmark: {config.get('benchmark_name', 'gguf_compare')}")
    print(f"Config: {config_path}")
    print(f"Metric: {metric.key} ({metric.unit}, {metric.direction} is better)")
    print(f"Runs: {runs}, warmup: {warmup_runs}, timeout: {timeout_s}s")

    backend_commands: Dict[str, str] = {}
    for backend in backends:
        if backend.get("enabled", True) is False:
            continue
        name = str(backend.get("name", "")).strip()
        if not name:
            raise ValueError("Each backend must have a non-empty 'name'")
        if not backend_is_selected(name, backend_filters):
            continue
        command_tmpl = str(backend.get("command", "")).strip()
        if not command_tmpl:
            raise ValueError(f"Backend '{name}' is missing 'command'")
        backend_commands[name] = format_command(command_tmpl, variables)

    if not backend_commands:
        requested = args.backend if args.backend else "<none>"
        print(
            f"No enabled backends matched --backend filter: {requested}",
            file=sys.stderr,
        )
        return 1

    if args.dry_run:
        print("\n[Dry run] Backend commands:")
        for name, command in backend_commands.items():
            print(f"- {name}: {command}")
        return 0

    result_rows: List[Dict[str, Any]] = []

    for backend in backends:
        if backend.get("enabled", True) is False:
            continue

        name = str(backend["name"])
        if name not in backend_commands:
            continue
        command = backend_commands[name]
        cwd_raw = str(backend.get("cwd", "."))
        cwd_path = (REPO_ROOT / cwd_raw) if not Path(cwd_raw).is_absolute() else Path(cwd_raw)
        env = os.environ.copy()
        for k, v in backend.get("env", {}).items():
            env[str(k)] = format_command(str(v), variables)

        metric_regexes = normalize_regexes(backend.get("metric_regexes"))
        if not metric_regexes:
            parse_section = backend.get("parse", {})
            metric_regexes = normalize_regexes(parse_section.get(metric.key))
        if not metric_regexes and not metric_fallback_to_wall_time:
            raise ValueError(
                f"Backend '{name}' has no metric regexes for '{metric.key}' and fallback is disabled."
            )

        aux_regexes: Dict[str, List[str]] = {}
        parse_section = backend.get("parse", {})
        aux_section = backend.get("aux_metrics", {})
        for key, value in aux_section.items():
            aux_regexes[str(key)] = normalize_regexes(value)
        for key, value in parse_section.items():
            if key == metric.key:
                continue
            if key not in aux_regexes:
                aux_regexes[str(key)] = normalize_regexes(value)

        print(f"\n== Backend: {name} ==")
        print(f"Command: {command}")
        print(f"CWD: {cwd_path}")

        for i in range(1, warmup_runs + 1):
            print(f"Warmup {i}/{warmup_runs}...")
            try:
                run_once(
                    backend_name=name,
                    command=command,
                    cwd=cwd_path,
                    timeout_s=timeout_s,
                    env=env,
                    metric_regexes=metric_regexes,
                    aux_regexes=aux_regexes,
                    metric_fallback_to_wall_time=metric_fallback_to_wall_time,
                )
            except subprocess.TimeoutExpired:
                if strict:
                    raise
                print("Warmup timeout (continuing because strict=false).")

        measured_runs: List[Dict[str, Any]] = []
        success_metrics: List[float] = []
        success_wall: List[float] = []
        aux_values: Dict[str, List[float]] = {}
        failed_runs = 0

        for i in range(1, runs + 1):
            print(f"Run {i}/{runs}...")
            try:
                record = run_once(
                    backend_name=name,
                    command=command,
                    cwd=cwd_path,
                    timeout_s=timeout_s,
                    env=env,
                    metric_regexes=metric_regexes,
                    aux_regexes=aux_regexes,
                    metric_fallback_to_wall_time=metric_fallback_to_wall_time,
                )
            except subprocess.TimeoutExpired:
                failed_runs += 1
                measured_runs.append(
                    {
                        "run_index": i,
                        "ok": False,
                        "error": f"timeout after {timeout_s}s",
                    }
                )
                if strict:
                    print("Timeout in strict mode; aborting benchmark.", file=sys.stderr)
                    return 2
                continue

            ok = (record["exit_code"] == 0) and (record["primary_metric"] is not None)
            measured_runs.append(
                {
                    "run_index": i,
                    "ok": ok,
                    "exit_code": record["exit_code"],
                    "wall_seconds": record["wall_seconds"],
                    "primary_metric": record["primary_metric"],
                    "aux_metrics": record["aux_metrics"],
                    "stdout_tail": (record["stdout"] or "")[-1200:],
                    "stderr_tail": (record["stderr"] or "")[-1200:],
                }
            )

            if ok:
                success_metrics.append(float(record["primary_metric"]))
                success_wall.append(float(record["wall_seconds"]))
                for key, val in record["aux_metrics"].items():
                    aux_values.setdefault(key, []).append(float(val))
            else:
                failed_runs += 1
                if strict:
                    print(
                        f"Backend '{name}' run {i} failed (exit={record['exit_code']}, metric={record['primary_metric']}).",
                        file=sys.stderr,
                    )
                    return 2

        summary = {
            "primary_metric": summarize(success_metrics),
            "wall_seconds": summarize(success_wall),
            "success_runs": len(success_metrics),
            "failed_runs": failed_runs,
            "aux_metrics": {k: summarize(v) for k, v in aux_values.items()},
        }
        status = "ok" if success_metrics else "failed"

        result_rows.append(
            {
                "name": name,
                "status": status,
                "command": command,
                "cwd": to_posix(cwd_path),
                "runs": measured_runs,
                "summary": summary,
            }
        )

    good = [
        row
        for row in result_rows
        if row["status"] == "ok"
        and row.get("summary", {}).get("primary_metric", {}).get("median") is not None
    ]
    best_median: Optional[float] = None
    if good:
        medians = [float(row["summary"]["primary_metric"]["median"]) for row in good]
        best_median = max(medians) if metric.direction == "higher" else min(medians)
        if best_median > 0:
            for row in good:
                med = float(row["summary"]["primary_metric"]["median"])
                if metric.direction == "higher":
                    row["summary"]["median_vs_best"] = med / best_median
                else:
                    row["summary"]["median_vs_best"] = best_median / med if med else 0.0

    now = datetime.now(timezone.utc)
    benchmark_result = {
        "benchmark": {
            "benchmark_name": str(config.get("benchmark_name", "gguf_compare")),
            "timestamp_utc": now.isoformat(),
            "hardware": str(config.get("hardware", "unknown")),
            "model_name": str(config.get("model", {}).get("name", "")),
            "model_path": str(config.get("model", {}).get("path", "")),
            "prompt": str(config.get("prompt", "")),
            "max_tokens": int(config.get("max_tokens", 256)),
            "runs": runs,
            "warmup_runs": warmup_runs,
            "timeout_seconds": timeout_s,
            "metric": {
                "key": metric.key,
                "unit": metric.unit,
                "direction": metric.direction,
            },
            "strict": strict,
        },
        "results": result_rows,
    }

    default_output_dir = config.get("output_dir", "benchmarks/results/gguf")
    out_dir = (REPO_ROOT / default_output_dir) if not Path(default_output_dir).is_absolute() else Path(default_output_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    stamp = now.strftime("%Y%m%d_%H%M%S")
    stem = output_stem(config)

    output_json = (
        Path(args.output_json)
        if args.output_json
        else out_dir / f"{stem}_{stamp}.json"
    )
    output_md = (
        Path(args.output_md)
        if args.output_md
        else out_dir / f"{stem}_{stamp}.md"
    )
    output_html = (
        Path(args.output_html)
        if args.output_html
        else out_dir / f"{stem}_{stamp}.html"
    )

    output_json = (REPO_ROOT / output_json) if not output_json.is_absolute() else output_json
    output_md = (REPO_ROOT / output_md) if not output_md.is_absolute() else output_md
    output_html = (REPO_ROOT / output_html) if not output_html.is_absolute() else output_html

    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_md.parent.mkdir(parents=True, exist_ok=True)
    output_html.parent.mkdir(parents=True, exist_ok=True)

    with output_json.open("w", encoding="utf-8") as f:
        json.dump(benchmark_result, f, indent=2)
    with output_md.open("w", encoding="utf-8") as f:
        f.write(generate_markdown_report(benchmark_result, metric))
    with output_html.open("w", encoding="utf-8") as f:
        f.write(generate_html_report(benchmark_result, metric))

    print("\nBenchmark completed.")
    print(f"- JSON: {output_json}")
    print(f"- Markdown: {output_md}")
    print(f"- HTML: {output_html}")

    if best_median is not None:
        print(f"- Best median {metric.key}: {best_median:.4f} {metric.unit}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
