### In-process boot (create engine/VM + evaluate trivial script), µs

Median of per-pass medians over 3 passes; p95 = median of per-pass p95s; spread = min..max of the per-pass medians.

| Engine | Variant | median µs | p95 µs | pass spread (median) | iterations/pass |
|---|---|---:|---:|---|---:|
| mlua Lua 5.4 | `new+eval` | 20.0 | 22.5 | 19.3..20.2 | 1000 |
| mlua Lua 5.5 | `new+eval` | 23.8 | 27.1 | 23.7..24.2 | 1000 |
| mlua LuaJIT | `new+eval` | 60.3 | 65.8 | 60.0..60.8 | 1000 |
| mlua Luau | `new+eval` | 41.7 | 46.2 | 41.2..41.8 | 1000 |
| mlua Luau + native codegen | `new+eval` | 78.3 | 87.4 | 78.1..79.3 | 1000 |
| rhai | `new+eval` | 76.3 | 82.9 | 75.2..79.0 | 1000 |
| rhai | `new_raw+eval` | 0.541 | 0.584 | 0.500..0.542 | 1000 |
| rquickjs (QuickJS) | `runtime+context_full+eval` | 45.5 | 51.3 | 45.3..47.0 | 1000 |
| boa_engine | `context+eval` | 79.3 | 440 | 79.2..84.3 | 1000 |
| rune | `context+compile+call` | 1,547 | 1,589 | 1,539..1,552 | 1000 |
| rune | `context_only` | 1,446 | 1,479 | 1,445..1,450 | 1000 |
| rune | `warm_context:compile+call` | 6.83 | 7.08 | 6.75..6.88 | 1000 |
| koto | `new+compile_and_run` | 17.9 | 19.5 | 16.7..18.2 | 1000 |
| wasmtime (Cranelift) | `engine+compile+instantiate+call` | 258 | 299 | 258..259 | 1000 |
| wasmtime (Cranelift) | `engine+deserialize+instantiate+call` | 19.3 | 23.2 | 15.4..20.5 | 1000 |
| wasmtime (Cranelift) | `engine_new_only` | 0.834 | 0.917 | 0.833..1.00 | 1000 |
| wasmtime (Cranelift) | `warm_engine:compile+instantiate+call` | 257 | 295 | 253..265 | 1000 |
| wasmtime (Cranelift) | `warm_engine:deserialize+instantiate+call` | 13.8 | 20.0 | 13.8..21.5 | 1000 |
| wasmtime (Cranelift) | `warm_instance_pre:instantiate+call` | 0.709 | 0.750 | 0.708..0.834 | 1000 |
| wasmtime (Cranelift) | `warm_module:instantiate+call` | 1.42 | 1.54 | 1.42..1.67 | 1000 |
| wasmi | `engine+compile+instantiate+call` | 5.42 | 5.67 | 5.33..6.04 | 1000 |
| wasmi | `warm_module:instantiate+call` | 0.750 | 0.833 | 0.750..0.875 | 1000 |
| pyo3 (CPython 3.13) | `first_initialize+eval` | 4,862 | 4,862 | 4,300..30,450 | 1 |
| pyo3 (CPython 3.13) | `warm_interp_attach+eval` | 2.08 | 2.29 | 2.08..2.29 | 1000 |
| starlark-rust | `globals+parse+eval` | 3.79 | 3.92 | 3.58..3.96 | 1000 |
| steel | `new+run` | 56,983 | 57,794 | 56,980..62,852 | 160 |
| steel | `new_base+run` | 25,191 | 25,546 | 25,174..26,357 | 376 |

### Whole-process wall time (hyperfine -N, warmup 5, 50 runs/pass, 3 passes) and stripped binary size

Process = spawn binary, create engine, evaluate trivial script, print result, exit. Median ms is the median of per-pass hyperfine medians; 'over native' subtracts the native hello-world median.

| Engine | binary | median ms | min ms | mean±σ ms (pass 1) | over native ms | stripped size |
|---|---|---:|---:|---|---:|---:|
| native Rust | `boot_native` | 1.25 | 1.17 | 1.25±0.05 | +0.00 | 0.34 MB |
| mlua Lua 5.4 | `boot_lua54` | 1.42 | 1.34 | 1.42±0.03 | +0.18 | 0.76 MB |
| mlua Lua 5.5 | `boot_lua55` | 1.42 | 1.34 | 1.42±0.04 | +0.17 | 0.75 MB |
| mlua LuaJIT | `boot_luajit` | 1.52 | 1.44 | 1.51±0.03 | +0.27 | 1.08 MB |
| mlua Luau | `boot_luau` | 2.15 | 2.05 | 2.12±0.04 | +0.91 | 1.81 MB |
| mlua Luau + native codegen | `boot_luau_jit` | 2.33 | 2.23 | 2.34±0.05 | +1.09 | 2.40 MB |
| rhai | `boot_rhai` | 1.57 | 1.49 | 1.57±0.04 | +0.32 | 3.29 MB |
| rquickjs (QuickJS) | `boot_rquickjs` | 1.45 | 1.36 | 1.45±0.02 | +0.20 | 1.38 MB |
| boa_engine | `boot_boa` | 1.89 | 1.80 | 1.91±0.05 | +0.65 | 10.32 MB |
| rune | `boot_rune` | 3.47 | 3.36 | 3.47±0.05 | +2.23 | 5.57 MB |
| koto | `boot_koto` | 2.15 | 2.04 | 2.14±0.06 | +0.91 | 2.06 MB |
| wasmtime (Cranelift) | `boot_wasmtime` | 3.03 | 2.63 | 3.06±0.13 | +1.79 | 11.96 MB |
| wasmi | `boot_wasmi` | 1.32 | 1.20 | 1.32±0.06 | +0.07 | 2.59 MB |
| pyo3 (CPython 3.13) | `boot_pyo3` | 7.90 | 7.63 | 7.93±0.16 | +6.65 | 0.40 MB |
| starlark-rust | `boot_starlark` | 2.70 | 2.61 | 2.71±0.05 | +1.45 | 6.18 MB |
| steel | `boot_steel` | 87.93 | 85.40 | 88.32±0.85 | +86.69 | 5.93 MB |
| wasmtime (Cranelift) (deserialize .cwasm) | `boot_wasmtime_precompiled` | 1.65 | 1.57 | 1.67±0.05 | +0.41 | 9.16 MB |

### Throughput: fib(n) recursive

| Engine | n | median ms | pass spread ms | ns/call | × native | × Lua 5.4 |
|---|---:|---:|---|---:|---:|---:|
| native Rust | 30 | 1.19 | 1.18..1.21 | 0.44 | 1.0 | 0.06 |
| wasmtime (Cranelift) | 30 | 3.06 | 2.84..3.25 | 1.14 | 2.6 | 0.15 |
| mlua LuaJIT | 30 | 4.12 | 4.02..4.19 | 1.53 | 3.5 | 0.20 |
| wasmi | 30 | 12.6 | 12.4..14.1 | 4.68 | 10.6 | 0.61 |
| mlua Luau + native codegen | 30 | 18.1 | 18.0..18.3 | 6.71 | 15.2 | 0.87 |
| mlua Lua 5.4 | 30 | 20.7 | 20.4..21.2 | 7.67 | 17.4 | 1.00 |
| mlua Lua 5.5 | 30 | 21.5 | 21.4..21.6 | 7.97 | 18.0 | 1.04 |
| mlua Luau | 30 | 36.2 | 35.7..37.1 | 13.43 | 30.4 | 1.75 |
| rquickjs (QuickJS) | 30 | 47.5 | 46.8..51.9 | 17.64 | 39.9 | 2.30 |
| pyo3 (CPython 3.13) | 30 | 52.6 | 52.0..58.9 | 19.53 | 44.2 | 2.54 |
| starlark-rust | 30 | 65.3 | 64.9..71.9 | 24.27 | 54.9 | 3.16 |
| rune | 30 | 88.5 | 88.3..93.9 | 32.87 | 74.4 | 4.28 |
| steel | 30 | 96.0 | 94.9..96.4 | 35.66 | 80.7 | 4.65 |
| boa_engine | 30 | 260 | 256..280 | 96.69 | 218.8 | 12.60 |
| koto | 30 | 267 | 263..275 | 99.13 | 224.3 | 12.92 |
| rhai | 30 | 618 | 608..622 | 229.37 | 519.0 | 29.89 |

### Throughput: loop summing 0..n-1

| Engine | n | median ms | pass spread ms | ns/iter | × native | × Lua 5.4 |
|---|---:|---:|---|---:|---:|---:|
| wasmtime (Cranelift) | 10,000,000 | 2.49 | 2.34..2.69 | 0.25 | 0.9 | 0.08 |
| native Rust | 10,000,000 | 2.91 | 2.90..2.94 | 0.29 | 1.0 | 0.10 |
| mlua LuaJIT | 10,000,000 | 5.40 | 5.36..5.50 | 0.54 | 1.9 | 0.18 |
| mlua Lua 5.5 | 10,000,000 | 17.7 | 17.7..17.7 | 1.77 | 6.1 | 0.58 |
| wasmi | 10,000,000 | 19.1 | 19.0..21.6 | 1.91 | 6.6 | 0.63 |
| mlua Lua 5.4 | 10,000,000 | 30.2 | 30.2..31.0 | 3.02 | 10.4 | 1.00 |
| mlua Luau + native codegen | 10,000,000 | 30.9 | 30.9..31.3 | 3.09 | 10.6 | 1.02 |
| mlua Luau | 10,000,000 | 31.4 | 28.1..31.8 | 3.14 | 10.8 | 1.04 |
| rquickjs (QuickJS) | 10,000,000 | 124 | 121..124 | 12.37 | 42.5 | 4.09 |
| pyo3 (CPython 3.13) | 10,000,000 | 157 | 151..164 | 15.66 | 53.8 | 5.18 |
| starlark-rust | 10,000,000 | 172 | 166..191 | 17.19 | 59.1 | 5.69 |
| boa_engine | 10,000,000 | 196 | 190..205 | 19.63 | 67.5 | 6.49 |
| koto | 10,000,000 | 277 | 275..308 | 27.72 | 95.3 | 9.17 |
| rhai | 10,000,000 | 368 | 366..392 | 36.81 | 126.6 | 12.18 |
| rune | 10,000,000 | 407 | 401..446 | 40.66 | 139.8 | 13.45 |
| steel | 10,000,000 | 438 | 421..439 | 43.82 | 150.7 | 14.50 |

### Throughput: build string from n pieces (`tostring(i)` joined)

| Engine | n | median ms | pass spread ms | ns/piece | × native | × Lua 5.4 |
|---|---:|---:|---|---:|---:|---:|
| native Rust | 100,000 | 0.786 | 0.746..0.810 | 7.86 | 1.0 | 0.11 |
| mlua LuaJIT | 100,000 | 1.83 | 1.78..1.86 | 18.30 | 2.3 | 0.25 |
| starlark-rust | 100,000 | 3.43 | 3.43..3.56 | 34.27 | 4.4 | 0.47 |
| pyo3 (CPython 3.13) | 100,000 | 3.44 | 3.29..3.50 | 34.39 | 4.4 | 0.47 |
| mlua Luau + native codegen | 100,000 | 5.02 | 4.91..5.25 | 50.23 | 6.4 | 0.69 |
| mlua Luau | 100,000 | 5.06 | 5.02..5.16 | 50.62 | 6.4 | 0.69 |
| rquickjs (QuickJS) | 100,000 | 5.15 | 5.14..5.20 | 51.52 | 6.6 | 0.70 |
| mlua Lua 5.5 | 100,000 | 7.02 | 6.90..7.04 | 70.16 | 8.9 | 0.96 |
| mlua Lua 5.4 | 100,000 | 7.33 | 7.26..7.62 | 73.28 | 9.3 | 1.00 |
| rune | 100,000 | 12.1 | 11.2..12.1 | 120.61 | 15.3 | 1.65 |
| rhai | 100,000 | 14.9 | 14.8..14.9 | 149.09 | 19.0 | 2.03 |
| koto | 100,000 | 17.6 | 17.3..18.3 | 176.46 | 22.5 | 2.41 |
| steel | 100,000 | 19.0 | 18.6..19.1 | 190.35 | 24.2 | 2.60 |
| boa_engine | 100,000 | 24.3 | 23.1..24.9 | 242.68 | 30.9 | 3.31 |

Not measured: wasmtime (Cranelift), wasmi (no string type in core WASM).

### Throughput: script calls Rust `add(acc,1)` n times

| Engine | n | median ms | pass spread ms | ns/call | × native | × Lua 5.4 |
|---|---:|---:|---|---:|---:|---:|
| native Rust | 1,000,000 | 0.700 | 0.694..0.707 | 0.70 | 1.0 | 0.04 |
| wasmtime (Cranelift) | 1,000,000 | 2.77 | 2.77..2.98 | 2.77 | 4.0 | 0.16 |
| wasmi | 1,000,000 | 10.2 | 9.50..11.3 | 10.17 | 14.5 | 0.58 |
| starlark-rust | 1,000,000 | 10.9 | 10.4..11.8 | 10.94 | 15.6 | 0.62 |
| mlua LuaJIT | 1,000,000 | 14.8 | 14.7..14.9 | 14.78 | 21.1 | 0.84 |
| mlua Lua 5.5 | 1,000,000 | 17.1 | 16.8..17.4 | 17.07 | 24.4 | 0.97 |
| mlua Lua 5.4 | 1,000,000 | 17.6 | 17.5..17.8 | 17.63 | 25.2 | 1.00 |
| mlua Luau | 1,000,000 | 21.8 | 21.4..22.1 | 21.77 | 31.1 | 1.23 |
| mlua Luau + native codegen | 1,000,000 | 22.3 | 22.3..23.8 | 22.34 | 31.9 | 1.27 |
| pyo3 (CPython 3.13) | 1,000,000 | 24.2 | 23.8..27.7 | 24.16 | 34.5 | 1.37 |
| steel | 1,000,000 | 40.9 | 40.6..41.0 | 40.91 | 58.4 | 2.32 |
| rune | 1,000,000 | 47.5 | 46.4..52.3 | 47.49 | 67.8 | 2.69 |
| rquickjs (QuickJS) | 1,000,000 | 51.0 | 50.9..51.7 | 51.03 | 72.9 | 2.90 |
| boa_engine | 1,000,000 | 67.2 | 65.4..71.5 | 67.24 | 96.1 | 3.81 |
| koto | 1,000,000 | 71.2 | 70.8..76.0 | 71.21 | 101.7 | 4.04 |
| rhai | 1,000,000 | 98.7 | 96.6..105 | 98.72 | 141.0 | 5.60 |

### Throughput: Rust calls script `inc(i)` n times

| Engine | n | median ms | pass spread ms | ns/call | × native | × Lua 5.4 |
|---|---:|---:|---|---:|---:|---:|
| native Rust | 100,000 | 0.069 | 0.069..0.071 | 0.69 | 1.0 | 0.03 |
| wasmtime (Cranelift) | 100,000 | 1.25 | 1.25..1.38 | 12.55 | 18.2 | 0.57 |
| mlua LuaJIT | 100,000 | 1.71 | 1.69..1.73 | 17.14 | 24.8 | 0.78 |
| starlark-rust | 100,000 | 1.94 | 1.84..2.00 | 19.42 | 28.1 | 0.89 |
| mlua Lua 5.5 | 100,000 | 2.12 | 2.04..2.18 | 21.20 | 30.7 | 0.97 |
| mlua Lua 5.4 | 100,000 | 2.19 | 2.18..2.33 | 21.93 | 31.8 | 1.00 |
| pyo3 (CPython 3.13) | 100,000 | 2.68 | 2.51..2.89 | 26.83 | 38.9 | 1.22 |
| mlua Luau | 100,000 | 2.72 | 2.69..2.73 | 27.17 | 39.4 | 1.24 |
| mlua Luau + native codegen | 100,000 | 2.86 | 2.81..2.98 | 28.58 | 41.4 | 1.30 |
| rquickjs (QuickJS) | 100,000 | 3.81 | 3.76..4.12 | 38.13 | 55.3 | 1.74 |
| wasmi | 100,000 | 3.94 | 3.91..4.43 | 39.45 | 57.2 | 1.80 |
| rune | 100,000 | 4.61 | 4.58..5.05 | 46.06 | 66.8 | 2.10 |
| boa_engine | 100,000 | 5.66 | 5.66..6.16 | 56.56 | 82.0 | 2.58 |
| koto | 100,000 | 7.08 | 7.05..7.46 | 70.75 | 102.5 | 3.23 |
| steel | 100,000 | 7.58 | 7.57..7.64 | 75.76 | 109.8 | 3.45 |
| rhai | 100,000 | 12.0 | 11.9..13.1 | 120.10 | 174.1 | 5.48 |

