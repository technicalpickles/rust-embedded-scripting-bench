# Local benchmarks: scripting engines embedded in Rust

Hands-on measurements run on 2026-10-27. Every number below came from a run of the code in this repo
(`run_all.sh` → `aggregate.py`). Nothing is estimated; where something wasn't measured it says so.

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
| Rhai | `rhai 1.27.0` (default features) | |
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

