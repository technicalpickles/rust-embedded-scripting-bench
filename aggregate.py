#!/usr/bin/env python3
"""Aggregate results/raw_bench_pass*.tsv, hyperfine_pass*.json, sizes.tsv into markdown tables.
Per (engine, metric): the reported value is the median of the per-pass medians; the spread
column is min..max of the per-pass medians."""
import glob, json, statistics, sys, os
os.chdir(os.path.join(os.path.dirname(os.path.abspath(__file__)), "results"))

ENGINES = ["native","lua54","lua55","luajit","luau","luau_jit","rhai","rquickjs","boa","rune","koto",
           "wasmtime","wasmi","pyo3","starlark","steel"]
LABEL = {"native":"native Rust","lua54":"mlua Lua 5.4","lua55":"mlua Lua 5.5","luajit":"mlua LuaJIT",
         "luau":"mlua Luau","luau_jit":"mlua Luau + native codegen","rhai":"rhai","rquickjs":"rquickjs (QuickJS)",
         "boa":"boa_engine","rune":"rune","koto":"koto","wasmtime":"wasmtime (Cranelift)","wasmi":"wasmi",
         "pyo3":"pyo3 (CPython 3.13)","starlark":"starlark-rust","steel":"steel"}

data = {}  # (engine, metric) -> list of dicts per pass
passes = sorted(glob.glob("raw_bench_pass*.tsv"))
for p in passes:
    for line in open(p):
        f = line.rstrip("\n").split("\t")
        if f[0] != "RESULT": continue
        _, eng, metric, n, med, p95, mn, unit, note = (f + [""])[:9]
        data.setdefault((eng, metric), []).append(dict(n=int(n), med=float(med), p95=float(p95), min=float(mn), unit=unit, note=note))

def agg(eng, metric):
    rows = data.get((eng, metric))
    if not rows: return None
    meds = [r["med"] for r in rows]
    return dict(n=rows[0]["n"], med=statistics.median(meds), lo=min(meds), hi=max(meds),
                p95=statistics.median([r["p95"] for r in rows]), unit=rows[0]["unit"], passes=len(rows), note=rows[0]["note"])

def fmt(x):
    if x >= 100: return f"{x:,.0f}"
    if x >= 10: return f"{x:.1f}"
    if x >= 1: return f"{x:.2f}"
    return f"{x:.3f}"

out = []
P = out.append

P(f"### In-process boot (create engine/VM + evaluate trivial script), µs\n")
P(f"Median of per-pass medians over {len(passes)} passes; p95 = median of per-pass p95s; spread = min..max of the per-pass medians.\n")
P("| Engine | Variant | median µs | p95 µs | pass spread (median) | iterations/pass |")
P("|---|---|---:|---:|---|---:|")
for e in ENGINES:
    for (eng, metric) in sorted(k for k in data if k[0] == e and k[1].startswith("boot:")):
        a = agg(eng, metric)
        P(f"| {LABEL[e]} | `{metric[5:]}` | {fmt(a['med'])} | {fmt(a['p95'])} | {fmt(a['lo'])}..{fmt(a['hi'])} | {a['n']} |")
P("")

# hyperfine
hf = {}
for p in sorted(glob.glob("hyperfine_pass*.json")):
    for r in json.load(open(p))["results"]:
        hf.setdefault(r["command"], []).append(r)
sizes = dict(l.split("\t") for l in open("sizes.tsv").read().split("\n") if l.strip())
native_hf = statistics.median([r["median"] for r in hf["./target/release/boot_native"]]) * 1e3
P(f"### Whole-process wall time (hyperfine -N, warmup 5, 50 runs/pass, {len(next(iter(hf.values())))} passes) and stripped binary size\n")
P("Process = spawn binary, create engine, evaluate trivial script, print result, exit. Median ms is the median of per-pass hyperfine medians; 'over native' subtracts the native hello-world median.\n")
P("| Engine | binary | median ms | min ms | mean±σ ms (pass 1) | over native ms | stripped size |")
P("|---|---|---:|---:|---|---:|---:|")
for cmd, rs in hf.items():
    b = cmd.split()[0].split("/")[-1]
    e = b.replace("boot_", "").replace("_precompiled", "")
    med = statistics.median([r["median"] for r in rs]) * 1e3
    mn = min(r["min"] for r in rs) * 1e3
    r1 = rs[0]
    lab = LABEL.get(e, e) + (" (deserialize .cwasm)" if "precompiled" in b else "")
    sz = int(sizes[b])
    P(f"| {lab} | `{b}` | {med:.2f} | {mn:.2f} | {r1['mean']*1e3:.2f}±{r1['stddev']*1e3:.2f} | {med-native_hf:+.2f} | {sz/1e6:.2f} MB |")
P("")

W = [("fib", "fib(n) recursive", "call"), ("loop_sum", "loop summing 0..n-1", "iter"),
     ("str_build", "build string from n pieces (`tostring(i)` joined)", "piece"),
     ("host_call", "script calls Rust `add(acc,1)` n times", "call"),
     ("call_from_rust", "Rust calls script `inc(i)` n times", "call")]
def fibcalls(n):
    # number of calls made by naive fib(n): 2*fib(n+1)-1
    a, b = 0, 1
    for _ in range(n + 1): a, b = b, a + b
    return 2 * a - 1
for metric, title, per in W:
    nat = agg("native", metric); lua = agg("lua54", metric)
    P(f"### Throughput: {title}\n")
    P(f"| Engine | n | median ms | pass spread ms | ns/{per} | × native | × Lua 5.4 |")
    P("|---|---:|---:|---|---:|---:|---:|")
    rows = []
    for e in ENGINES:
        a = agg(e, metric)
        if not a: continue
        rows.append((a["med"], e, a))
    for _, e, a in sorted(rows):
        units = fibcalls(a["n"]) if metric == "fib" else a["n"]
        nsper = a["med"] * 1e6 / units
        xn = f"{a['med']/nat['med']:.1f}" if nat else "-"
        xl = f"{a['med']/lua['med']:.2f}" if lua else "-"
        P(f"| {LABEL[e]} | {a['n']:,} | {fmt(a['med'])} | {fmt(a['lo'])}..{fmt(a['hi'])} | {nsper:.2f} | {xn} | {xl} |")
    skipped = [e for e in ENGINES if not agg(e, metric)]
    if skipped: P(f"\nNot measured: {', '.join(LABEL[e] for e in skipped)} (no string type in core WASM).")
    P("")
print("\n".join(out))
