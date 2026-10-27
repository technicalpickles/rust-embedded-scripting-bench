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

