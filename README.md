# rust-embedded-scripting-bench

Benchmarks for scripting languages you can embed in a Rust host, measuring both sides of the tradeoff:

- **Boot**: what a short-lived CLI pays (whole-process wall time via hyperfine, in-process engine create+eval+drop, stripped binary size)
- **Throughput**: what a long-running app pays (fib(30), a 10M integer loop, string building, script→Rust and Rust→script call overhead)

Full results and methodology: [RESULTS.md](RESULTS.md). Measured on an Apple M5, rustc 1.96, 2026-10-27.

## Engines

| Crate | Engine |
|---|---|
| `mlua` | Lua 5.4, Lua 5.5, LuaJIT, Luau, Luau + native codegen |
| `rquickjs` | QuickJS-NG |
| `boa_engine` | Boa (JS) |
| `rhai` | Rhai (AST interpreter; `grain` not enabled) |
| `rune` | Rune |
| `koto` | Koto |
| `starlark` | starlark-rust |
| `steel-core` | Steel (Scheme) |
| `wasmtime` | Wasmtime (Cranelift), from bytes and from precompiled `.cwasm` |
| `wasmi` | Wasmi |
| `pyo3` | CPython 3.13 |

Plus a native Rust baseline. V8/deno_core isn't included (build weight).

## Headline (fib(30), × Lua 5.4; process overhead over a 1.25 ms native hello-world)

| Engine | fib(30) | Boot overhead |
|---|---:|---:|
| wasmtime | 0.15 | +1.79 ms (+0.41 precompiled) |
| LuaJIT | 0.20 | +0.27 ms |
| wasmi | 0.61 | +0.07 ms |
| Lua 5.4 | 1.00 | +0.18 ms |
| Luau | 1.75 | +0.91 ms |
| rquickjs | 2.30 | +0.20 ms |
| CPython | 2.54 | +6.65 ms |
| Starlark | 3.16 | +1.45 ms |
| Rune | 4.28 | +2.23 ms |
| Steel | 4.7 | +86.7 ms |
| Boa | 12.6 | +0.65 ms |
| Koto | 12.9 | +0.91 ms |
| Rhai | 29.9 | +0.32 ms |

## Layout

- `harness/`: timing helpers
- `engines/<name>/`: one crate per engine, each with a `bench_<name>` and a `boot_<name>` binary
- `shared/`: code shared across the mlua flavors
- `scripts/`: the benchmark script in each language
- `results/`: raw TSV, hyperfine JSON/markdown, binary sizes from the run in RESULTS.md

## Running

Needs `hyperfine` and, for the pyo3 engine, a CPython 3.13 with a shared `libpython`.

```bash
./run_all.sh            # build each engine, in-process benches, hyperfine, sizes
python3 aggregate.py    # build the tables
```

`SKIP_BUILD=1`, `SKIP_INPROC=1`, `PASS=n` and `HF_RUNS=n` tune `run_all.sh`. The mlua flavors are built one crate at a time because their features can't unify in one build.
