# Local benchmarks: scripting engines embedded in Rust

Hands-on measurements run on 2026-10-27. Every number below came from a run of the code in this repo
(`run_all.sh` → `aggregate.py`). Nothing is estimated; where something wasn't measured it says so.

## Headline findings

All figures are from the tables below (median of 3 passes).

- **For short-lived CLI processes, boot cost mostly disappears into process spawn.** A native hello-world takes 1.25 ms wall on this Mac.
  Lua 5.4 adds +0.18 ms, rquickjs +0.20, wasmi +0.07, LuaJIT +0.27, Rhai +0.32, wasmtime with a precompiled `.cwasm` +0.41, Boa +0.65, Luau/Koto +0.91.
  The outliers: Starlark +1.45, wasmtime compiling from wasm bytes +1.79, Rune +2.23, CPython via pyo3 +6.65 (7.9 ms total), and **Steel +86.7 ms** (88 ms total, from compiling its Scheme prelude at engine creation).
- **In-process engine creation** (create + eval trivial + drop): Starlark 3.8 µs, wasmi 5.4 µs, Koto 18 µs, Lua 5.4 20 µs, Luau 42 µs, rquickjs 45 µs,
  LuaJIT 60 µs, Rhai `Engine::new()` 76 µs, Boa 79 µs, wasmtime 258 µs (Cranelift compile), Rune 1.5 ms, Steel 57 ms.
  Rune's cost is almost all `Context::with_default_modules()` (1.45 ms); with a reused Context, compile+call is 6.8 µs.
  wasmtime: compile from bytes 258 µs vs `Module::deserialize` 19 µs vs `InstancePre::instantiate` 0.7 µs.
- **Throughput tiers (fib(30), × Lua 5.4):** wasmtime 0.15, LuaJIT 0.20, wasmi 0.61, Luau+codegen 0.87, Lua 5.4 1.0, Lua 5.5 1.04, Luau interpreter 1.75,
  QuickJS 2.3, CPython 2.5, Starlark 3.2, Rune 4.3, Steel 4.7, Boa 12.6, Koto 12.9, **Rhai 29.9** (618 ms).
  On the integer loop, Rhai, Rune, Steel are 12 to 15x slower than Lua 5.4, Koto 9x, Boa 6.5x.
- **Luau native codegen only helped recursion** (fib 36 → 18 ms); loop, string and host-call workloads were unchanged, and it costs ~37 µs more boot and +0.6 MB.
  The Luau interpreter was slower than PUC Lua 5.4 on fib (1.75x) and on par on the loop.
- **Lua 5.5 beat 5.4 on the integer loop** (17.7 vs 30.2 ms) and was on par elsewhere.
- **Host-function call cost (per script iteration incl. loop):** wasmtime 2.8 ns, wasmi 10 ns, Starlark 11 ns, LuaJIT 15 ns, Lua 17 to 18 ns, Luau 22 ns,
  CPython 24 ns, Steel 41 ns, Rune 47 ns, QuickJS 51 ns, Boa 67 ns, Koto 71 ns, Rhai 99 ns.
  **Rust→script call:** wasmtime 12.6 ns, LuaJIT 17 ns, Starlark 19 ns, Lua 21 to 22 ns, CPython 27 ns, Luau 27 ns, QuickJS 38 ns, wasmi 39 ns, Rune 46 ns, Boa 57 ns, Koto 71 ns, Steel 76 ns, Rhai 120 ns (the last four include a by-name lookup per call).
- **String building** is where the interpreters bunch up: everything except LuaJIT (1.8 ms) and native (0.8 ms) lands in 3.4 to 24 ms for 100k pieces; CPython and Starlark (3.4 ms) beat Lua 5.4 (7.3 ms).
- **Binary size (stripped `boot_*`, no LTO):** native 0.34 MB, Lua 5.4 0.76 MB, LuaJIT 1.08 MB, QuickJS 1.38 MB, Luau 1.81 MB (2.40 with codegen), Koto 2.06 MB, wasmi 2.59 MB,
  Rhai 3.29 MB, Rune 5.57 MB, Steel 5.93 MB, Starlark 6.18 MB, Boa 10.3 MB, wasmtime 12.0 MB. pyo3 is 0.40 MB plus a dynamically loaded 17.5 MB libpython and the stdlib.

## Machine

- Apple M5 (10 cores: 4 performance + 6 efficiency, per `sysctl hw.perflevel*`), 24 GB RAM
- macOS 26.6.2 (build 25G83), Darwin 25.6.0
- rustc 1.96.0 (ac68faa20 2026-05-25), cargo 1.96.0
- hyperfine at /opt/homebrew/bin/hyperfine
- CPython 3.13.14 (mise install, python-build-standalone, `libpython3.13.dylib` linked dynamically)

## Crate versions (resolved in Cargo.lock)

| Engine | Crate(s) | Underlying VM version |
|---|---|---|
| Lua 5.4 / 5.5 / LuaJIT / Luau | `mlua 0.12.2`, `mlua-sys 0.13.0`, features `vendored` + flavor | Lua 5.4.9 and 5.5.1 (`lua-src 551.0.2`); LuaJIT 2.1 @ c6ffc14 (`luajit-src 210.7.4`); Luau 0.740 (`luau0-src 0.22.0+luau740`) |
| Luau native codegen | `mlua` feature `luau-jit` (mlua enables codegen by default when built with it and calls `luau_codegen_compile` on every loaded chunk) | same Luau 0.740 |
| Rhai | `rhai 1.27.0` (default features) | **AST-walking interpreter, NOT Grain.** Default features are only `std` + `ahash/runtime-rng`; the `grain` bytecode-VM feature was not enabled (verified in `engines/rhai/Cargo.toml` and rhai-1.27.0's `Cargo.toml` `[features]`), and the bench runs `engine.compile()` → `call_fn` on the AST. Grain numbers were not measured. |
| QuickJS | `rquickjs 0.14.0` / `rquickjs-sys 0.14.0` (default features) | quickjs-ng 0.16 (per bundled `quickjs.h`) |
| Boa | `boa_engine 0.22.0` | |
| Rune | `rune 0.14.2` | |
| Koto | `koto 0.16.1` | |
| Wasmtime | `wasmtime 49.0.4` (default features: Cranelift, parallel compilation, cache, etc.) | see "dropped / substituted" below |
| wasmi | `wasmi 2.0.0` (default features) | |
| CPython | `pyo3 0.30.1`, feature `auto-initialize` | CPython 3.13.14 |
| Starlark | `starlark 0.14.2` (Meta's starlark-rust) | |
| Steel | `steel-core 0.8.3` (default features) | |
| WAT assembler | `wat 1.262.0` (build-dependency only; WAT → wasm happens in `build.rs`, outside every timed region) | |

## Engines dropped or substituted

- **None dropped.** All 12 requested engine families built and ran, plus Lua 5.5 as a bonus (mlua 0.12 supports it).
- **wasmtime 50.0.0 → 49.0.4.** 50.0.0 (latest on crates.io) refuses to build on rustc 1.96: `cranelift-*@0.137.0 requires rustc 1.97.0`. Pinned `wasmtime = "49"` and it built cleanly.
- V8 / deno_core excluded per instructions.

## Methodology

**Layout.** One Cargo workspace (`resolver = "3"`, edition 2024). Shared `harness` lib crate does timing and prints
`RESULT\tengine\tmetric\tn\tmedian\tp95\tmin\tunit` lines. One crate per engine under `engines/`, each with two binaries:
`bench_<engine>` (in-process boot + throughput) and `boot_<engine>` (tiny: create engine, eval trivial script, print, exit).
The four mlua flavors are separate crates that `#[path]`-include one shared source file (`shared/mlua_bench.rs`), and each crate is
built with its own `cargo build -p` because mlua's Lua-flavor features are mutually exclusive.
Release profile: `opt-level = 3`, `strip = true`, `debug = false`, no LTO, default codegen-units, `panic = unwind`.

**Scripts** live in `scripts/` (`bench.lua` shared by all four Lua flavors, `bench.js` shared by rquickjs and boa, `bench.rhai`, `bench.rn`,
`bench.koto`, `bench.py`, `bench.star`, `bench.scm`, `bench.wat`). Each script is loaded once; the workload functions are then called from
Rust with the size as an argument, and every return value is checked against a Rust-computed expected value (all checks passed).

**A) Boot.**
- *In-process:* closure that creates a fresh engine/VM/context, evaluates a trivial script (`1+1` or equivalent; for WASM, instantiate and
  call an exported `answer()` returning 42), returns the integer. One warmup call, then 1000 timed iterations with `Instant` (or fewer
  if a 10 s budget runs out; minimum 20). The engine is dropped inside the timed closure, so **teardown is included**.
  Reported: median and p95 in µs.
- *Whole process:* `hyperfine -N --warmup 5 --runs 50` over every `boot_*` binary, plus a native Rust hello-world (`boot_native`)
  that prints `black_box(1)+1`. This is the "short-lived CLI command" number.
- *Binary size:* size of the stripped `boot_*` release binary.

**B) Throughput** (engine warm, script loaded). One untimed warmup + correctness run, then 5 timed runs; median reported.
- `fib(30)` naive recursive (2,692,537 calls). No engine needed scaling down, so all ran at n=30.
- `loop_sum(10_000_000)`: sum of 0..n-1 in a script loop.
- `str_build(100_000)`: convert each i in 0..n to a string and concatenate (488,890 chars total), using each language's idiomatic builder (see deviations).
- `host_call(1_000_000)`: script loop doing `acc = add(acc, 1)` where `add` is a Rust function registered with the engine. **Includes the script's loop overhead**; compare against `loop_sum` per-iteration cost to estimate pure call cost.
- `call_from_rust(100_000)`: Rust loop calling the script function `inc(i)` (returns i+1) once per iteration, summing results.
- Native Rust baseline: same algorithms; inputs passed through `std::hint::black_box`; `loop_sum` does `s += black_box(i)` to stop LLVM from closed-forming/vectorizing; `add`/`inc` are `#[inline(never)]` and called through a `black_box`ed fn pointer.

**Passes.** The whole suite (in-process + hyperfine + sizes) was run 3 times back to back. Tables report the **median of the three
per-pass medians**, and the min..max of the per-pass medians as "pass spread".
A discarded `bench_native` run precedes each pass: in an earlier attempt (not reported) the first benchmark of a cold pass ran ~2x slow
(native fib 2.8 ms vs a steady 1.19 ms), consistent with CPU clock ramp or E-core scheduling; the warm-up fixed it.

## Deviations from strict semantic equivalence

- **Number types differ.** Lua 5.4/5.5, Rhai, Rune, Koto, Starlark, Steel, Python and WASM use integers. LuaJIT, Luau, QuickJS and Boa use
  IEEE doubles. Starlark stores ints >2^31 as heap bigints, so `loop_sum` (result ~5e13) allocates in its hot loop. Python ints are arbitrary precision objects.
- **String building is idiomatic, not identical:** Lua `t[i+1]=tostring(i)` + `table.concat`; JS `parts.push(String(i))` + `join('')`;
  Python/Starlark `parts.append(str(i))` + `"".join`; Rhai `s += i.to_string()` (in-place append; Rhai has no join on arrays by default);
  Rune `s.push_str(i.to_string())` (Rune 0.14's `Vec` has no `join`); Koto `parts.push '{i}'` + `iterator.to_string parts` + `size` (size counts graphemes);
  Steel `cons` list of `number->string`, `reverse`, `apply string-append`. Native: `write!(s, "{i}")`.
- **WASM has no string workload** (core WASM has no string type; a fair version needs a guest allocator/runtime). Not measured for wasmtime/wasmi.
- **Rust→script call handle:** mlua, pyo3, boa, wasmtime, wasmi call a pre-resolved function handle. rquickjs restores a `Persistent<Function>`
  inside `ctx.with` per call. Rhai (`call_fn`), Rune (`vm.call([name])`), Koto (`call_exported_function`) and Steel
  (`call_function_by_name_with_args`) look the function up by name on each call, because that is their documented API; their
  `call_from_rust` numbers include that lookup.
- **Host-function signatures:** i64 in/out for mlua, Rhai, Rune, wasmtime, wasmi, pyo3, Starlark; `isize` for Steel; `f64` for rquickjs and Boa (JS numbers); Koto `KNumber`.
- **Boot "trivial script" granularity varies:** Rune's boot compiles a `pub fn main() { 1 + 1 }` unit and calls it (it has no eval); Steel runs `(+ 1 1)`; WASM instantiates and calls `answer()`.
- **Luau** uses mlua's default compiler settings (`optimization_level: 1`); `--!optimize 2` was not tried. Luau native codegen used mlua defaults (no type info, inliner untouched).
- **Rhai `new_raw`** creates an engine without any packages; its 0.5 µs number likely also benefits from Rhai's optimizer constant-folding `1+1`. Treat it as "floor", not a fair boot comparison.
- **CPython can only initialize once per process**, so its in-process boot is one sample per pass (4.3 ms, 4.9 ms, and 30.5 ms on the first, cold-cache pass). The hyperfine process number is the meaningful one. `warm_interp_attach+eval` is just attaching to an already-running interpreter.
- **wasmtime `engine_new_only` / `warm_*` variants** run after the process already created one `Engine`, so process-global one-time setup isn't in them. The process-level `boot_wasmtime` vs `boot_wasmtime_precompiled` numbers capture that.

## Results

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


## Caveats

- Single machine, single sitting, no core pinning (macOS doesn't expose it). P vs E core placement is up to the scheduler; pass spreads show the noise.
- Microbenchmarks. fib/loop stress call and arithmetic dispatch; real scripts that spend time in host APIs, tables/objects, or GC will rank differently.
- The native baseline is "reasonable Rust", not "maximally optimized": `loop_sum` is held to ~1 cycle/iter by `black_box`, which is why wasmtime's Cranelift loop came out at 0.9x native. LLVM partially transforms native `fib` recursion, so its 0.44 ns/call is optimistic.
- Throughput runs are 5 timed runs after one warmup per pass; JIT engines (LuaJIT, Luau codegen, Cranelift) are fully warmed by then for these tiny kernels.
- Binary sizes are the `boot_*` binaries with `strip = true` and no LTO/`opt-level="z"`; LTO or size-optimized builds would shrink them. `boot_pyo3` is small because `libpython3.13.dylib` (17.5 MB on disk, plus the stdlib directory) is loaded dynamically and not counted.
- Process wall time on macOS has a ~1.2 ms floor (dyld + process spawn) even for hello-world, so the "over native" column is the useful one for comparing engines.
- Boot numbers include engine teardown (drop), which a CLI that just exits would skip.

## Raw output

Raw per-pass TSV (`RESULT engine metric n median p95 min unit note`; ms for throughput, µs for boot), followed by hyperfine markdown per pass.
Files: `results/raw_bench_pass{1,2,3}.tsv`, `results/hyperfine_pass{1,2,3}.{json,md,txt}`, `results/sizes.tsv`.

### Pass 1: in-process

```
== bench_native
RESULT	native	fib	30	1.206	1.242	1.181	ms	
RESULT	native	loop_sum	10000000	2.944	2.981	2.838	ms	
RESULT	native	str_build	100000	0.786	0.790	0.737	ms	
RESULT	native	host_call	1000000	0.694	0.720	0.689	ms	
RESULT	native	call_from_rust	100000	0.071	0.074	0.071	ms	
== bench_lua54
RESULT	lua54	boot:new+eval	1000	20.042	23.542	19.208	us	n=iterations
RESULT	lua54	fib	30	21.173	21.240	21.123	ms	
RESULT	lua54	loop_sum	10000000	30.204	30.515	30.031	ms	
RESULT	lua54	str_build	100000	7.257	8.241	6.996	ms	
RESULT	lua54	host_call	1000000	17.456	17.536	16.951	ms	
RESULT	lua54	call_from_rust	100000	2.330	2.342	2.322	ms	
== bench_lua55
RESULT	lua55	boot:new+eval	1000	23.792	26.791	22.416	us	n=iterations
RESULT	lua55	fib	30	21.640	22.180	21.607	ms	
RESULT	lua55	loop_sum	10000000	17.666	17.779	17.380	ms	
RESULT	lua55	str_build	100000	6.901	7.590	6.699	ms	
RESULT	lua55	host_call	1000000	16.838	17.296	16.397	ms	
RESULT	lua55	call_from_rust	100000	2.178	2.248	2.160	ms	
== bench_luajit
RESULT	luajit	boot:new+eval	1000	60.333	65.750	57.959	us	n=iterations
RESULT	luajit	fib	30	4.023	4.030	4.009	ms	
RESULT	luajit	loop_sum	10000000	5.362	5.401	5.344	ms	
RESULT	luajit	str_build	100000	1.856	5.638	1.756	ms	
RESULT	luajit	host_call	1000000	14.781	15.238	13.721	ms	
RESULT	luajit	call_from_rust	100000	1.727	1.728	1.717	ms	
== bench_luau
RESULT	luau	boot:new+eval	1000	41.709	46.167	40.875	us	n=iterations
RESULT	luau	fib	30	36.167	36.243	35.742	ms	
RESULT	luau	loop_sum	10000000	31.834	32.353	31.706	ms	
RESULT	luau	str_build	100000	5.160	6.436	4.649	ms	
RESULT	luau	host_call	1000000	21.406	21.670	21.245	ms	
RESULT	luau	call_from_rust	100000	2.729	2.796	2.726	ms	
== bench_luau_jit
RESULT	luau_jit	boot:new+eval	1000	79.333	87.375	77.084	us	n=iterations
RESULT	luau_jit	fib	30	18.342	18.354	18.284	ms	
RESULT	luau_jit	loop_sum	10000000	31.344	31.471	16.156	ms	
RESULT	luau_jit	str_build	100000	5.245	6.675	4.622	ms	
RESULT	luau_jit	host_call	1000000	23.787	26.534	23.343	ms	
RESULT	luau_jit	call_from_rust	100000	2.981	3.011	2.972	ms	
== bench_rhai
RESULT	rhai	boot:new+eval	1000	79.042	83.333	74.667	us	n=iterations
RESULT	rhai	boot:new_raw+eval	1000	0.542	0.584	0.500	us	n=iterations
RESULT	rhai	fib	30	621.920	659.962	610.973	ms	
RESULT	rhai	loop_sum	10000000	392.311	405.599	385.019	ms	
RESULT	rhai	str_build	100000	14.909	15.444	14.685	ms	
RESULT	rhai	host_call	1000000	105.331	105.755	102.214	ms	
RESULT	rhai	call_from_rust	100000	13.139	13.273	13.033	ms	
== bench_rquickjs
RESULT	rquickjs	boot:runtime+context_full+eval	1000	47.042	55.750	44.833	us	n=iterations
RESULT	rquickjs	fib	30	51.933	52.078	51.683	ms	
RESULT	rquickjs	loop_sum	10000000	121.252	126.401	119.453	ms	
RESULT	rquickjs	str_build	100000	5.152	5.520	5.003	ms	
RESULT	rquickjs	host_call	1000000	51.716	52.810	51.263	ms	
RESULT	rquickjs	call_from_rust	100000	4.118	4.147	3.950	ms	
== bench_boa
RESULT	boa	boot:context+eval	1000	84.292	440.458	77.750	us	n=iterations
RESULT	boa	fib	30	280.104	282.558	271.851	ms	
RESULT	boa	loop_sum	10000000	205.301	209.293	193.488	ms	
RESULT	boa	str_build	100000	24.268	24.751	23.808	ms	
RESULT	boa	host_call	1000000	71.458	71.844	69.905	ms	
RESULT	boa	call_from_rust	100000	6.163	6.295	6.039	ms	
== bench_rune
RESULT	rune	boot:context+compile+call	1000	1551.542	1610.000	1503.208	us	n=iterations
RESULT	rune	boot:warm_context:compile+call	1000	6.875	7.833	6.625	us	n=iterations
RESULT	rune	boot:context_only	1000	1445.625	1481.208	1395.083	us	n=iterations
RESULT	rune	fib	30	93.890	94.041	93.501	ms	
RESULT	rune	loop_sum	10000000	445.999	466.381	436.250	ms	
RESULT	rune	str_build	100000	12.141	12.210	11.914	ms	
RESULT	rune	host_call	1000000	52.332	53.515	51.080	ms	
RESULT	rune	call_from_rust	100000	5.051	5.072	4.841	ms	
== bench_koto
RESULT	koto	boot:new+compile_and_run	1000	17.875	18.416	17.542	us	n=iterations
RESULT	koto	fib	30	275.146	287.455	269.629	ms	
RESULT	koto	loop_sum	10000000	307.784	309.980	287.139	ms	
RESULT	koto	str_build	100000	18.292	19.076	17.713	ms	
RESULT	koto	host_call	1000000	75.989	76.395	74.731	ms	
RESULT	koto	call_from_rust	100000	7.455	7.503	7.395	ms	
== bench_wasmtime
INFO	wasmtime	wasm_bytes=294	cwasm_bytes=51272
RESULT	wasmtime	boot:engine+compile+instantiate+call	1000	258.041	312.750	209.250	us	n=iterations
RESULT	wasmtime	boot:engine+deserialize+instantiate+call	1000	20.542	25.875	17.500	us	n=iterations
RESULT	wasmtime	boot:engine_new_only	1000	1.000	1.042	0.916	us	n=iterations
RESULT	wasmtime	boot:warm_engine:compile+instantiate+call	1000	265.375	310.125	218.958	us	n=iterations
RESULT	wasmtime	boot:warm_engine:deserialize+instantiate+call	1000	21.500	23.458	16.583	us	n=iterations
RESULT	wasmtime	boot:warm_module:instantiate+call	1000	1.667	1.750	1.583	us	n=iterations
RESULT	wasmtime	boot:warm_instance_pre:instantiate+call	1000	0.834	0.917	0.791	us	n=iterations
RESULT	wasmtime	fib	30	3.063	3.074	3.062	ms	
RESULT	wasmtime	loop_sum	10000000	2.690	2.701	2.689	ms	
SKIP	wasmtime	str_build	unsupported
RESULT	wasmtime	host_call	1000000	2.982	2.989	2.978	ms	
RESULT	wasmtime	call_from_rust	100000	1.375	1.388	1.367	ms	
== bench_wasmi
RESULT	wasmi	boot:engine+compile+instantiate+call	1000	6.042	6.375	5.708	us	n=iterations
RESULT	wasmi	boot:warm_module:instantiate+call	1000	0.875	0.958	0.750	us	n=iterations
RESULT	wasmi	fib	30	14.071	14.970	13.658	ms	
RESULT	wasmi	loop_sum	10000000	21.623	22.265	17.285	ms	
SKIP	wasmi	str_build	unsupported
RESULT	wasmi	host_call	1000000	11.301	12.567	10.927	ms	
RESULT	wasmi	call_from_rust	100000	4.427	4.516	4.337	ms	
== bench_pyo3
RESULT	pyo3	boot:first_initialize+eval	1	30450.500	30450.500	30450.500	us	single sample (CPython init is once per process)
RESULT	pyo3	boot:warm_interp_attach+eval	1000	2.292	2.458	2.125	us	interpreter already initialized; not a cold boot
RESULT	pyo3	fib	30	58.940	60.173	57.374	ms	
RESULT	pyo3	loop_sum	10000000	163.897	176.171	158.272	ms	
RESULT	pyo3	str_build	100000	3.439	3.503	3.434	ms	
RESULT	pyo3	host_call	1000000	27.706	29.075	27.544	ms	
RESULT	pyo3	call_from_rust	100000	2.893	2.893	2.777	ms	
== bench_starlark
RESULT	starlark	boot:globals+parse+eval	1000	3.958	4.084	3.792	us	n=iterations
RESULT	starlark	fib	30	71.895	74.315	71.176	ms	
RESULT	starlark	loop_sum	10000000	191.257	193.663	183.399	ms	
RESULT	starlark	str_build	100000	3.426	3.564	3.399	ms	
RESULT	starlark	host_call	1000000	10.941	11.033	10.644	ms	
RESULT	starlark	call_from_rust	100000	1.997	2.021	1.944	ms	
== bench_steel
RESULT	steel	boot:new+run	160	62851.958	68611.833	57252.000	us	n=iterations
RESULT	steel	boot:new_base+run	376	26357.334	29031.292	25053.875	us	n=iterations
RESULT	steel	fib	30	96.352	97.251	95.673	ms	
RESULT	steel	loop_sum	10000000	438.156	443.411	423.208	ms	
RESULT	steel	str_build	100000	18.552	19.485	18.430	ms	
RESULT	steel	host_call	1000000	40.913	42.132	40.137	ms	
RESULT	steel	call_from_rust	100000	7.572	7.685	7.430	ms	
```

### Pass 2: in-process

```
== bench_native
RESULT	native	fib	30	1.190	1.193	1.182	ms	
RESULT	native	loop_sum	10000000	2.908	2.958	2.861	ms	
RESULT	native	str_build	100000	0.810	0.876	0.780	ms	
RESULT	native	host_call	1000000	0.700	0.702	0.464	ms	
RESULT	native	call_from_rust	100000	0.069	0.072	0.069	ms	
== bench_lua54
RESULT	lua54	boot:new+eval	1000	19.333	21.750	18.417	us	n=iterations
RESULT	lua54	fib	30	20.664	21.097	20.609	ms	
RESULT	lua54	loop_sum	10000000	30.227	30.696	30.188	ms	
RESULT	lua54	str_build	100000	7.328	8.226	6.912	ms	
RESULT	lua54	host_call	1000000	17.813	18.073	17.721	ms	
RESULT	lua54	call_from_rust	100000	2.181	2.193	2.170	ms	
== bench_lua55
RESULT	lua55	boot:new+eval	1000	23.708	27.125	22.583	us	n=iterations
RESULT	lua55	fib	30	21.453	23.060	20.811	ms	
RESULT	lua55	loop_sum	10000000	17.652	17.787	14.402	ms	
RESULT	lua55	str_build	100000	7.045	8.566	6.644	ms	
RESULT	lua55	host_call	1000000	17.383	17.952	17.351	ms	
RESULT	lua55	call_from_rust	100000	2.120	2.158	2.111	ms	
== bench_luajit
RESULT	luajit	boot:new+eval	1000	60.792	65.875	58.375	us	n=iterations
RESULT	luajit	fib	30	4.123	4.147	4.114	ms	
RESULT	luajit	loop_sum	10000000	5.397	5.480	5.342	ms	
RESULT	luajit	str_build	100000	1.784	5.586	1.723	ms	
RESULT	luajit	host_call	1000000	14.689	15.108	14.215	ms	
RESULT	luajit	call_from_rust	100000	1.714	1.720	1.705	ms	
== bench_luau
RESULT	luau	boot:new+eval	1000	41.209	45.792	40.208	us	n=iterations
RESULT	luau	fib	30	35.698	36.018	35.125	ms	
RESULT	luau	loop_sum	10000000	28.116	31.800	25.759	ms	
RESULT	luau	str_build	100000	5.062	6.474	4.780	ms	
RESULT	luau	host_call	1000000	22.110	23.026	21.938	ms	
RESULT	luau	call_from_rust	100000	2.692	2.774	2.681	ms	
== bench_luau_jit
RESULT	luau_jit	boot:new+eval	1000	78.334	87.750	75.000	us	n=iterations
RESULT	luau_jit	fib	30	18.045	18.090	17.674	ms	
RESULT	luau_jit	loop_sum	10000000	30.913	30.957	30.891	ms	
RESULT	luau_jit	str_build	100000	4.909	6.274	4.707	ms	
RESULT	luau_jit	host_call	1000000	22.340	22.485	21.374	ms	
RESULT	luau_jit	call_from_rust	100000	2.858	3.037	2.764	ms	
== bench_rhai
RESULT	rhai	boot:new+eval	1000	75.209	82.208	71.375	us	n=iterations
RESULT	rhai	boot:new_raw+eval	1000	0.541	0.625	0.416	us	n=iterations
RESULT	rhai	fib	30	608.196	614.492	606.537	ms	
RESULT	rhai	loop_sum	10000000	365.520	366.194	365.118	ms	
RESULT	rhai	str_build	100000	14.925	15.009	14.552	ms	
RESULT	rhai	host_call	1000000	98.716	99.147	98.388	ms	
RESULT	rhai	call_from_rust	100000	12.010	12.129	11.911	ms	
== bench_rquickjs
RESULT	rquickjs	boot:runtime+context_full+eval	1000	45.500	51.292	44.167	us	n=iterations
RESULT	rquickjs	fib	30	47.505	48.435	46.945	ms	
RESULT	rquickjs	loop_sum	10000000	124.014	124.710	122.703	ms	
RESULT	rquickjs	str_build	100000	5.137	6.091	4.997	ms	
RESULT	rquickjs	host_call	1000000	51.028	51.299	50.924	ms	
RESULT	rquickjs	call_from_rust	100000	3.813	4.064	3.581	ms	
== bench_boa
RESULT	boa	boot:context+eval	1000	79.250	428.958	75.959	us	n=iterations
RESULT	boa	fib	30	260.353	264.475	258.897	ms	
RESULT	boa	loop_sum	10000000	196.299	203.010	191.036	ms	
RESULT	boa	str_build	100000	24.898	24.949	24.191	ms	
RESULT	boa	host_call	1000000	67.241	67.391	66.942	ms	
RESULT	boa	call_from_rust	100000	5.655	5.663	5.478	ms	
== bench_rune
RESULT	rune	boot:context+compile+call	1000	1539.250	1581.708	1483.584	us	n=iterations
RESULT	rune	boot:warm_context:compile+call	1000	6.750	7.042	6.542	us	n=iterations
RESULT	rune	boot:context_only	1000	1445.000	1477.625	1391.917	us	n=iterations
RESULT	rune	fib	30	88.503	88.928	88.301	ms	
RESULT	rune	loop_sum	10000000	406.635	409.724	401.401	ms	
RESULT	rune	str_build	100000	12.061	12.350	11.573	ms	
RESULT	rune	host_call	1000000	47.490	47.620	47.173	ms	
RESULT	rune	call_from_rust	100000	4.606	4.725	4.535	ms	
== bench_koto
RESULT	koto	boot:new+compile_and_run	1000	16.667	19.500	16.166	us	n=iterations
RESULT	koto	fib	30	266.900	267.398	264.976	ms	
RESULT	koto	loop_sum	10000000	277.193	277.577	274.959	ms	
RESULT	koto	str_build	100000	17.285	17.467	17.183	ms	
RESULT	koto	host_call	1000000	71.207	71.975	71.014	ms	
RESULT	koto	call_from_rust	100000	7.075	7.123	7.045	ms	
== bench_wasmtime
INFO	wasmtime	wasm_bytes=294	cwasm_bytes=51272
RESULT	wasmtime	boot:engine+compile+instantiate+call	1000	258.583	298.708	191.833	us	n=iterations
RESULT	wasmtime	boot:engine+deserialize+instantiate+call	1000	15.375	22.250	14.833	us	n=iterations
RESULT	wasmtime	boot:engine_new_only	1000	0.834	0.917	0.791	us	n=iterations
RESULT	wasmtime	boot:warm_engine:compile+instantiate+call	1000	253.458	289.500	186.125	us	n=iterations
RESULT	wasmtime	boot:warm_engine:deserialize+instantiate+call	1000	13.833	17.750	13.166	us	n=iterations
RESULT	wasmtime	boot:warm_module:instantiate+call	1000	1.417	1.541	1.333	us	n=iterations
RESULT	wasmtime	boot:warm_instance_pre:instantiate+call	1000	0.709	0.750	0.666	us	n=iterations
RESULT	wasmtime	fib	30	3.246	3.440	2.800	ms	
RESULT	wasmtime	loop_sum	10000000	2.492	2.505	2.488	ms	
SKIP	wasmtime	str_build	unsupported
RESULT	wasmtime	host_call	1000000	2.766	2.781	2.761	ms	
RESULT	wasmtime	call_from_rust	100000	1.255	1.257	1.250	ms	
== bench_wasmi
RESULT	wasmi	boot:engine+compile+instantiate+call	1000	5.417	5.666	5.166	us	n=iterations
RESULT	wasmi	boot:warm_module:instantiate+call	1000	0.750	0.792	0.666	us	n=iterations
RESULT	wasmi	fib	30	12.424	12.775	11.968	ms	
RESULT	wasmi	loop_sum	10000000	19.070	23.355	19.047	ms	
SKIP	wasmi	str_build	unsupported
RESULT	wasmi	host_call	1000000	10.168	10.520	9.927	ms	
RESULT	wasmi	call_from_rust	100000	3.912	3.923	3.894	ms	
== bench_pyo3
RESULT	pyo3	boot:first_initialize+eval	1	4861.500	4861.500	4861.500	us	single sample (CPython init is once per process)
RESULT	pyo3	boot:warm_interp_attach+eval	1000	2.084	2.292	1.958	us	interpreter already initialized; not a cold boot
RESULT	pyo3	fib	30	52.585	52.927	51.523	ms	
RESULT	pyo3	loop_sum	10000000	156.587	161.496	154.013	ms	
RESULT	pyo3	str_build	100000	3.502	3.530	3.436	ms	
RESULT	pyo3	host_call	1000000	24.163	24.191	23.674	ms	
RESULT	pyo3	call_from_rust	100000	2.513	2.664	2.501	ms	
== bench_starlark
RESULT	starlark	boot:globals+parse+eval	1000	3.583	3.708	3.458	us	n=iterations
RESULT	starlark	fib	30	65.343	65.396	64.910	ms	
RESULT	starlark	loop_sum	10000000	171.938	175.905	170.313	ms	
RESULT	starlark	str_build	100000	3.558	3.621	3.319	ms	
RESULT	starlark	host_call	1000000	10.353	10.470	10.330	ms	
RESULT	starlark	call_from_rust	100000	1.942	1.956	1.904	ms	
== bench_steel
RESULT	steel	boot:new+run	176	56980.292	57794.000	56447.959	us	n=iterations
RESULT	steel	boot:new_base+run	397	25174.500	25545.625	24743.542	us	n=iterations
RESULT	steel	fib	30	96.025	96.139	95.798	ms	
RESULT	steel	loop_sum	10000000	439.014	452.281	430.701	ms	
RESULT	steel	str_build	100000	19.082	19.176	18.357	ms	
RESULT	steel	host_call	1000000	40.981	41.909	40.777	ms	
RESULT	steel	call_from_rust	100000	7.644	7.818	7.568	ms	
```

### Pass 3: in-process

```
== bench_native
RESULT	native	fib	30	1.183	1.216	1.182	ms	
RESULT	native	loop_sum	10000000	2.896	2.907	2.870	ms	
RESULT	native	str_build	100000	0.746	0.776	0.736	ms	
RESULT	native	host_call	1000000	0.707	0.713	0.697	ms	
RESULT	native	call_from_rust	100000	0.069	0.069	0.069	ms	
== bench_lua54
RESULT	lua54	boot:new+eval	1000	20.209	22.459	19.542	us	n=iterations
RESULT	lua54	fib	30	20.393	22.129	20.189	ms	
RESULT	lua54	loop_sum	10000000	30.985	32.563	30.157	ms	
RESULT	lua54	str_build	100000	7.617	9.504	6.902	ms	
RESULT	lua54	host_call	1000000	17.626	17.907	17.357	ms	
RESULT	lua54	call_from_rust	100000	2.193	2.197	2.175	ms	
== bench_lua55
RESULT	lua55	boot:new+eval	1000	24.250	27.500	22.333	us	n=iterations
RESULT	lua55	fib	30	21.437	21.582	21.055	ms	
RESULT	lua55	loop_sum	10000000	17.703	17.800	17.674	ms	
RESULT	lua55	str_build	100000	7.016	7.884	6.851	ms	
RESULT	lua55	host_call	1000000	17.072	17.410	16.993	ms	
RESULT	lua55	call_from_rust	100000	2.036	2.054	2.030	ms	
== bench_luajit
RESULT	luajit	boot:new+eval	1000	60.041	65.375	57.166	us	n=iterations
RESULT	luajit	fib	30	4.186	4.197	4.172	ms	
RESULT	luajit	loop_sum	10000000	5.497	5.558	5.306	ms	
RESULT	luajit	str_build	100000	1.830	5.646	1.772	ms	
RESULT	luajit	host_call	1000000	14.927	15.266	14.766	ms	
RESULT	luajit	call_from_rust	100000	1.692	1.754	1.679	ms	
== bench_luau
RESULT	luau	boot:new+eval	1000	41.750	46.833	41.000	us	n=iterations
RESULT	luau	fib	30	37.143	38.843	35.755	ms	
RESULT	luau	loop_sum	10000000	31.354	31.774	27.676	ms	
RESULT	luau	str_build	100000	5.016	6.220	4.656	ms	
RESULT	luau	host_call	1000000	21.765	21.874	21.371	ms	
RESULT	luau	call_from_rust	100000	2.717	2.729	2.695	ms	
== bench_luau_jit
RESULT	luau_jit	boot:new+eval	1000	78.125	85.500	75.667	us	n=iterations
RESULT	luau_jit	fib	30	18.058	18.075	17.765	ms	
RESULT	luau_jit	loop_sum	10000000	30.913	30.953	30.906	ms	
RESULT	luau_jit	str_build	100000	5.023	6.498	4.676	ms	
RESULT	luau_jit	host_call	1000000	22.300	23.399	21.739	ms	
RESULT	luau_jit	call_from_rust	100000	2.805	2.809	2.768	ms	
== bench_rhai
RESULT	rhai	boot:new+eval	1000	76.292	82.875	71.083	us	n=iterations
RESULT	rhai	boot:new_raw+eval	1000	0.500	0.583	0.458	us	n=iterations
RESULT	rhai	fib	30	617.599	619.226	616.981	ms	
RESULT	rhai	loop_sum	10000000	368.134	368.448	367.777	ms	
RESULT	rhai	str_build	100000	14.792	14.966	14.612	ms	
RESULT	rhai	host_call	1000000	96.561	96.995	96.466	ms	
RESULT	rhai	call_from_rust	100000	11.928	11.963	11.849	ms	
== bench_rquickjs
RESULT	rquickjs	boot:runtime+context_full+eval	1000	45.333	49.000	44.250	us	n=iterations
RESULT	rquickjs	fib	30	46.767	46.989	46.705	ms	
RESULT	rquickjs	loop_sum	10000000	123.706	124.028	122.130	ms	
RESULT	rquickjs	str_build	100000	5.199	5.231	5.176	ms	
RESULT	rquickjs	host_call	1000000	50.949	51.140	50.906	ms	
RESULT	rquickjs	call_from_rust	100000	3.763	3.773	3.578	ms	
== bench_boa
RESULT	boa	boot:context+eval	1000	79.333	441.917	74.250	us	n=iterations
RESULT	boa	fib	30	255.811	256.380	254.523	ms	
RESULT	boa	loop_sum	10000000	189.660	192.471	181.964	ms	
RESULT	boa	str_build	100000	23.093	23.324	22.884	ms	
RESULT	boa	host_call	1000000	65.369	66.036	65.198	ms	
RESULT	boa	call_from_rust	100000	5.656	5.941	5.610	ms	
== bench_rune
RESULT	rune	boot:context+compile+call	1000	1546.833	1589.334	1500.791	us	n=iterations
RESULT	rune	boot:warm_context:compile+call	1000	6.833	7.083	6.625	us	n=iterations
RESULT	rune	boot:context_only	1000	1449.916	1478.916	1410.417	us	n=iterations
RESULT	rune	fib	30	88.317	89.116	88.125	ms	
RESULT	rune	loop_sum	10000000	401.402	406.888	398.488	ms	
RESULT	rune	str_build	100000	11.227	11.699	10.937	ms	
RESULT	rune	host_call	1000000	46.421	46.808	46.384	ms	
RESULT	rune	call_from_rust	100000	4.585	4.759	4.530	ms	
== bench_koto
RESULT	koto	boot:new+compile_and_run	1000	18.208	19.625	16.333	us	n=iterations
RESULT	koto	fib	30	262.635	263.020	261.340	ms	
RESULT	koto	loop_sum	10000000	274.962	276.612	273.246	ms	
RESULT	koto	str_build	100000	17.646	17.970	17.551	ms	
RESULT	koto	host_call	1000000	70.824	71.178	70.596	ms	
RESULT	koto	call_from_rust	100000	7.046	7.055	7.020	ms	
== bench_wasmtime
INFO	wasmtime	wasm_bytes=294	cwasm_bytes=51272
RESULT	wasmtime	boot:engine+compile+instantiate+call	1000	258.041	291.541	185.208	us	n=iterations
RESULT	wasmtime	boot:engine+deserialize+instantiate+call	1000	19.292	23.167	15.500	us	n=iterations
RESULT	wasmtime	boot:engine_new_only	1000	0.833	0.875	0.750	us	n=iterations
RESULT	wasmtime	boot:warm_engine:compile+instantiate+call	1000	256.583	295.209	191.958	us	n=iterations
RESULT	wasmtime	boot:warm_engine:deserialize+instantiate+call	1000	13.792	20.042	13.375	us	n=iterations
RESULT	wasmtime	boot:warm_module:instantiate+call	1000	1.417	1.500	1.333	us	n=iterations
RESULT	wasmtime	boot:warm_instance_pre:instantiate+call	1000	0.708	0.750	0.666	us	n=iterations
RESULT	wasmtime	fib	30	2.840	3.541	2.814	ms	
RESULT	wasmtime	loop_sum	10000000	2.341	2.393	2.323	ms	
SKIP	wasmtime	str_build	unsupported
RESULT	wasmtime	host_call	1000000	2.769	3.055	2.594	ms	
RESULT	wasmtime	call_from_rust	100000	1.255	1.266	1.249	ms	
== bench_wasmi
RESULT	wasmi	boot:engine+compile+instantiate+call	1000	5.334	5.542	5.125	us	n=iterations
RESULT	wasmi	boot:warm_module:instantiate+call	1000	0.750	0.833	0.666	us	n=iterations
RESULT	wasmi	fib	30	12.603	13.472	12.199	ms	
RESULT	wasmi	loop_sum	10000000	18.963	18.987	15.347	ms	
SKIP	wasmi	str_build	unsupported
RESULT	wasmi	host_call	1000000	9.503	10.150	9.322	ms	
RESULT	wasmi	call_from_rust	100000	3.945	3.955	3.930	ms	
== bench_pyo3
RESULT	pyo3	boot:first_initialize+eval	1	4299.667	4299.667	4299.667	us	single sample (CPython init is once per process)
RESULT	pyo3	boot:warm_interp_attach+eval	1000	2.084	2.250	1.917	us	interpreter already initialized; not a cold boot
RESULT	pyo3	fib	30	52.037	52.255	51.654	ms	
RESULT	pyo3	loop_sum	10000000	150.913	151.856	145.633	ms	
RESULT	pyo3	str_build	100000	3.286	3.384	3.249	ms	
RESULT	pyo3	host_call	1000000	23.803	24.284	23.698	ms	
RESULT	pyo3	call_from_rust	100000	2.683	3.260	2.666	ms	
== bench_starlark
RESULT	starlark	boot:globals+parse+eval	1000	3.792	3.917	3.667	us	n=iterations
RESULT	starlark	fib	30	64.925	68.852	64.425	ms	
RESULT	starlark	loop_sum	10000000	166.087	174.063	162.994	ms	
RESULT	starlark	str_build	100000	3.427	3.658	3.331	ms	
RESULT	starlark	host_call	1000000	11.802	12.264	10.264	ms	
RESULT	starlark	call_from_rust	100000	1.843	1.913	1.833	ms	
== bench_steel
RESULT	steel	boot:new+run	176	56983.166	57655.625	56444.000	us	n=iterations
RESULT	steel	boot:new_base+run	397	25191.416	25470.583	24812.917	us	n=iterations
RESULT	steel	fib	30	94.891	98.611	94.408	ms	
RESULT	steel	loop_sum	10000000	421.178	433.485	419.717	ms	
RESULT	steel	str_build	100000	19.035	19.721	18.676	ms	
RESULT	steel	host_call	1000000	40.633	41.713	40.502	ms	
RESULT	steel	call_from_rust	100000	7.576	7.601	7.368	ms	
```

### Pass 1: hyperfine

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `./target/release/boot_native` | 1.3 ± 0.0 | 1.2 | 1.4 | 1.00 |
| `./target/release/boot_lua54` | 1.4 ± 0.0 | 1.4 | 1.5 | 1.14 ± 0.05 |
| `./target/release/boot_lua55` | 1.4 ± 0.0 | 1.3 | 1.5 | 1.13 ± 0.05 |
| `./target/release/boot_luajit` | 1.5 ± 0.0 | 1.4 | 1.6 | 1.21 ± 0.05 |
| `./target/release/boot_luau` | 2.1 ± 0.0 | 2.1 | 2.2 | 1.69 ± 0.07 |
| `./target/release/boot_luau_jit` | 2.3 ± 0.0 | 2.2 | 2.5 | 1.87 ± 0.08 |
| `./target/release/boot_rhai` | 1.6 ± 0.0 | 1.5 | 1.7 | 1.26 ± 0.06 |
| `./target/release/boot_rquickjs` | 1.5 ± 0.0 | 1.4 | 1.5 | 1.16 ± 0.05 |
| `./target/release/boot_boa` | 1.9 ± 0.0 | 1.9 | 2.0 | 1.52 ± 0.07 |
| `./target/release/boot_rune` | 3.5 ± 0.0 | 3.4 | 3.6 | 2.77 ± 0.11 |
| `./target/release/boot_koto` | 2.1 ± 0.1 | 2.0 | 2.3 | 1.71 ± 0.08 |
| `./target/release/boot_wasmtime` | 3.1 ± 0.1 | 2.7 | 3.3 | 2.44 ± 0.14 |
| `./target/release/boot_wasmi` | 1.3 ± 0.1 | 1.2 | 1.6 | 1.06 ± 0.06 |
| `./target/release/boot_pyo3` | 7.9 ± 0.2 | 7.7 | 8.4 | 6.33 ± 0.27 |
| `./target/release/boot_starlark` | 2.7 ± 0.1 | 2.6 | 2.9 | 2.16 ± 0.09 |
| `./target/release/boot_steel` | 88.3 ± 0.8 | 86.5 | 90.2 | 70.44 ± 2.77 |
| `./target/release/boot_wasmtime_precompiled results/bench.cwasm` | 1.7 ± 0.1 | 1.6 | 1.9 | 1.33 ± 0.07 |


### Pass 2: hyperfine

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `./target/release/boot_native` | 1.3 ± 0.0 | 1.2 | 1.4 | 1.00 |
| `./target/release/boot_lua54` | 1.5 ± 0.0 | 1.4 | 1.6 | 1.16 ± 0.06 |
| `./target/release/boot_lua55` | 1.4 ± 0.0 | 1.4 | 1.5 | 1.13 ± 0.05 |
| `./target/release/boot_luajit` | 1.6 ± 0.1 | 1.5 | 1.8 | 1.24 ± 0.07 |
| `./target/release/boot_luau` | 2.2 ± 0.1 | 2.1 | 2.4 | 1.72 ± 0.08 |
| `./target/release/boot_luau_jit` | 2.3 ± 0.1 | 2.3 | 2.5 | 1.86 ± 0.08 |
| `./target/release/boot_rhai` | 1.6 ± 0.0 | 1.5 | 1.7 | 1.25 ± 0.06 |
| `./target/release/boot_rquickjs` | 1.5 ± 0.0 | 1.4 | 1.6 | 1.16 ± 0.05 |
| `./target/release/boot_boa` | 1.9 ± 0.0 | 1.8 | 1.9 | 1.49 ± 0.06 |
| `./target/release/boot_rune` | 3.5 ± 0.0 | 3.4 | 3.6 | 2.77 ± 0.11 |
| `./target/release/boot_koto` | 2.2 ± 0.1 | 2.1 | 2.3 | 1.71 ± 0.08 |
| `./target/release/boot_wasmtime` | 3.0 ± 0.2 | 2.6 | 3.3 | 2.42 ± 0.16 |
| `./target/release/boot_wasmi` | 1.3 ± 0.0 | 1.2 | 1.4 | 1.05 ± 0.05 |
| `./target/release/boot_pyo3` | 7.9 ± 0.2 | 7.6 | 8.6 | 6.31 ± 0.29 |
| `./target/release/boot_starlark` | 2.7 ± 0.1 | 2.6 | 2.9 | 2.17 ± 0.10 |
| `./target/release/boot_steel` | 87.7 ± 0.8 | 85.8 | 90.5 | 69.73 ± 2.78 |
| `./target/release/boot_wasmtime_precompiled results/bench.cwasm` | 1.7 ± 0.1 | 1.6 | 1.8 | 1.32 ± 0.07 |


### Pass 3: hyperfine

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `./target/release/boot_native` | 1.2 ± 0.0 | 1.2 | 1.4 | 1.00 |
| `./target/release/boot_lua54` | 1.4 ± 0.0 | 1.3 | 1.6 | 1.14 ± 0.05 |
| `./target/release/boot_lua55` | 1.4 ± 0.0 | 1.3 | 1.5 | 1.13 ± 0.05 |
| `./target/release/boot_luajit` | 1.5 ± 0.0 | 1.4 | 1.6 | 1.21 ± 0.05 |
| `./target/release/boot_luau` | 2.2 ± 0.0 | 2.1 | 2.3 | 1.73 ± 0.07 |
| `./target/release/boot_luau_jit` | 2.3 ± 0.1 | 2.2 | 2.6 | 1.88 ± 0.10 |
| `./target/release/boot_rhai` | 1.5 ± 0.0 | 1.5 | 1.6 | 1.24 ± 0.05 |
| `./target/release/boot_rquickjs` | 1.5 ± 0.0 | 1.4 | 1.5 | 1.17 ± 0.05 |
| `./target/release/boot_boa` | 1.9 ± 0.1 | 1.8 | 2.2 | 1.55 ± 0.08 |
| `./target/release/boot_rune` | 3.5 ± 0.1 | 3.4 | 3.6 | 2.78 ± 0.11 |
| `./target/release/boot_koto` | 2.2 ± 0.0 | 2.1 | 2.3 | 1.73 ± 0.07 |
| `./target/release/boot_wasmtime` | 3.0 ± 0.1 | 2.7 | 3.3 | 2.43 ± 0.15 |
| `./target/release/boot_wasmi` | 1.3 ± 0.0 | 1.2 | 1.5 | 1.04 ± 0.05 |
| `./target/release/boot_pyo3` | 7.9 ± 0.2 | 7.7 | 8.5 | 6.35 ± 0.28 |
| `./target/release/boot_starlark` | 2.7 ± 0.0 | 2.6 | 2.8 | 2.17 ± 0.09 |
| `./target/release/boot_steel` | 88.0 ± 1.0 | 85.4 | 89.8 | 70.66 ± 2.75 |
| `./target/release/boot_wasmtime_precompiled results/bench.cwasm` | 1.7 ± 0.1 | 1.6 | 1.8 | 1.34 ± 0.07 |


### Binary sizes (bytes, stripped)

```
boot_boa	10323712
boot_koto	2059456
boot_lua54	756848
boot_lua55	753376
boot_luajit	1084592
boot_luau	1811104
boot_luau_jit	2397360
boot_native	341616
boot_pyo3	398000
boot_rhai	3292304
boot_rquickjs	1376848
boot_rune	5571776
boot_starlark	6183888
boot_steel	5934608
boot_wasmi	2587920
boot_wasmtime	11960928
boot_wasmtime_precompiled	9160944
bench_boa	10357792
bench_koto	2093264
bench_lua54	774144
bench_lua55	787184
bench_luajit	1118464
bench_luau	1845072
bench_luau_jit	2431344
bench_native	376032
bench_pyo3	452464
bench_rhai	3342240
bench_rquickjs	1447632
bench_rune	5724176
bench_starlark	6268896
bench_steel	5951712
bench_wasmi	2621568
bench_wasmtime	11995424
```

### `otool -L boot_pyo3`

```
target/release/boot_pyo3:
	@rpath/libpython3.13.dylib (compatibility version 3.13.0, current version 3.13.0)
	/usr/lib/libiconv.2.dylib (compatibility version 7.0.0, current version 7.0.0)
	/usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1359.0.0)
```
