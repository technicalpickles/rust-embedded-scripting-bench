
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
