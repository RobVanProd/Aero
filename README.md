<div align="center">
  <h1>Aero v0.6.0</h1>
  <p><strong>The First Complete AI-Native Systems Language</strong></p>
  <a href="https://github.com/RobVanProd/Aero/stargazers">
    <img src="https://img.shields.io/github/stars/RobVanProd/Aero?style=social" alt="GitHub stars">
  </a>
  <a href="https://opensource.org/licenses/MIT">
    <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="MIT License">
  </a>
  <a href="https://github.com/RobVanProd/Aero/actions/workflows/ci.yml">
    <img src="https://github.com/RobVanProd/Aero/actions/workflows/ci.yml/badge.svg" alt="CI Status">
  </a>
</div>

**Rust-level safety • C-level speed • Python-level ergonomics** for AI and numerical computing.

Aero solves the two-language problem by providing a single, memory-safe systems language that delivers production-grade deep learning performance without Python or garbage collection overhead.

## 🚀 Live Ecosystem

- **[Interactive WebAssembly Playground](https://github.com/RobVanProd/Aero/tree/main/playground)** – Compile and run Aero code directly in your browser  
- **[Benchmarking Dashboard](https://github.com/RobVanProd/AeroNum/tree/main/benchmarks/dashboard)** – Live performance telemetry  
- **[Full Documentation](https://github.com/RobVanProd/Aero/tree/main/docs)** – “NumPy to AeroNum in 10 minutes” and API reference  

## ⚡ Performance Highlights

- **≥1.4× faster** than PyTorch on end-to-end GPT-2 mini-transformer training (1047 vs 748 tokens/sec)  
- **≥5.3× GPU acceleration** on 4096×4096 matrix multiplication with automatic `.to("cuda")` dispatch  
- **Zero garbage collection pauses** – full ownership model with compile-time safety  

## 🧠 Why Aero Wins

1. **Zero-Cost Abstractions** – Neural networks compile to native code with no runtime overhead  
2. **Memory Safety by Construction** – Ownership and borrowing eliminate use-after-free and data races  
3. **Familiar Ergonomics** – `Sequential`, `Dense`, `to("cuda")`, `save()`/`load()` feel like PyTorch  

## 📦 Quick Start

```bash
git clone https://github.com/RobVanProd/Aero.git
cd Aero
cargo build --release
export PATH="$PWD/target/release:$PATH"

aero new my_ai_project
cd my_ai_project
aero add aeronum aeronn
aero run
```

Try the flagship example directly in the Interactive Playground:

```aero
use aeronum::Array;
use aeronn::{Transformer, Sequential};

fn main() {
    let mut model = Transformer::new(layers: 6, dim: 384, heads: 6);
    model.to("cuda");
    // Train at native speed...
}
```

## 🗺️ Roadmap to v1.0.0 (Q2–Q3 2026)

- Distributed Training (Multi-GPU / multi-node) – Q2 2026
- INT8 / FP8 Quantization – Q2 2026
- Enhanced Compiler Diagnostics – Q2 2026
- Formal Language Specification – Q3 2026
- Kernel Fusion & Advanced Graph Compilation – Q3 2026
- Native Profiler & Flame Graphs – Q3 2026
- Central aero-pkg Registry (registry.aero) – Q3 2026

## License
MIT © RobVanProd and contributors. See LICENSE for details.
