<div align="center">
  <h1>Aero v1.0.0</h1>
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

- **[Interactive WebAssembly Playground](https://github.com/RobVanProd/AeroNum/tree/main/playground)** – Compile and run Aero code directly in your browser  
- **[Benchmarking Dashboard](https://github.com/RobVanProd/AeroNum/tree/main/benchmarks/dashboard)** – Live performance telemetry  
- **[Full Documentation](https://github.com/RobVanProd/AeroNum/tree/main/docs)** – "NumPy to AeroNum in 10 minutes" and API reference  

## ⚡ Performance Highlights

- **≥1.4× faster** than PyTorch on end-to-end GPT-2 mini-transformer training (1047 vs 748 tokens/sec)  
- **≥5.3× GPU acceleration** on 4096×4096 matrix multiplication with automatic `.to("cuda")` dispatch  
- **Near-linear multi-GPU scaling** via native NCCL/MPI distributed training (up to 8 GPUs)  
- **Zero garbage collection pauses** – full ownership model with compile-time safety  

## 🧠 Why Aero Wins

1. **Zero-Cost Abstractions** – Neural networks compile to native code with no runtime overhead  
2. **Memory Safety by Construction** – Ownership and borrowing eliminate use-after-free and data races  
3. **Familiar Ergonomics** – `Sequential`, `Dense`, `to("cuda")`, `save()`/`load()` feel like PyTorch  
4. **Distributed by Default** – Native DataParallel and ModelParallel with zero-copy NCCL communication  

## 📦 Quick Start

```bash
git clone https://github.com/RobVanProd/Aero.git
cd Aero
cargo build --release
export PATH="$PWD/target/release:$PATH"

# Initialize a new project scaffold
aero init my_app
cd my_app

# Compile + run
aero run src/main.aero

# Type-check only (no codegen)
aero check src/main.aero

# Generate Markdown API docs from source
aero doc src/main.aero -o main.md

# Profile compilation pipeline and export trace JSON
aero profile src/main.aero -o trace.json

# Language server for editor integration (stdio)
aero lsp
```

Try the flagship example directly in the Interactive Playground:

```aero
use aeronum::Array;
use aeronn::{Transformer, Sequential};

fn main() {
    let mut model = Transformer::new(layers: 6, dim: 384, heads: 6);
    model.to("distributed", 4);  // Scale across 4 GPUs
    // Train at native speed...
}
```

## 🛠️ Compiler Features (v1.0.0)

| Category | Features |
|----------|----------|
| **Type System** | Static typing, generics, trait bounds, where clauses |
| **Memory** | Ownership, move semantics, shared & mutable references, borrow checker |
| **Data Types** | Structs, enums, arrays, tuples, strings, pattern matching |
| **Control Flow** | Functions, if/else, while/for loops, break/continue, closures |
| **Modules** | `mod`/`use` imports, `pub` visibility, multi-file projects |
| **Codegen** | LLVM IR backend with optimization passes |
| **CLI** | `aero build`, `aero run`, `aero check`, `aero test`, `aero fmt`, `aero doc`, `aero profile`, `aero init`, `aero lsp` |
| **LSP** | Syntax diagnostics, completion, hover, go-to-definition, document symbols |
| **Docs & Profiling** | Markdown API generation (`aero doc`), compilation stage timing + trace export (`aero profile`) |
| **Diagnostics** | Colored errors, source snippets, "did you mean?" suggestions |

## 🗺️ Roadmap (v1.1.0+)

- INT8 / FP8 Quantization Interfaces
- Kernel Fusion & Advanced Graph Compilation
- Central aero-pkg Registry (registry.aero)
- Formal Language Specification

## License
MIT © RobVanProd and contributors. See LICENSE for details.
