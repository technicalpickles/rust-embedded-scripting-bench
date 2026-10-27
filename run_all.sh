#!/usr/bin/env bash
# Build every engine crate separately (mlua flavors can't share one feature-unified build),
# run the in-process benches, then hyperfine the boot binaries and record binary sizes.
set -euo pipefail
cd "$(dirname "$0")"
OUT=results
mkdir -p "$OUT"
ENGINES="native lua54 lua55 luajit luau luau_jit rhai rquickjs boa rune koto wasmtime wasmi pyo3 starlark steel"
pkg() { case $1 in native) echo native;; lua*|luau*) echo mlua_$1;; *) echo e_$1;; esac; }

if [[ "${SKIP_BUILD:-0}" != 1 ]]; then
  for e in $ENGINES; do
    cargo build --release -p "$(pkg $e)" 2>&1 | tail -1
  done
fi
./target/release/bench_wasmtime "$OUT/bench.cwasm"

if [[ "${SKIP_INPROC:-0}" != 1 ]]; then
  # Discarded warm-up: the first benchmark in a cold pass ran ~2x slow (CPU clock ramp /
  # core scheduling), which mostly hit the native baseline. Spin the CPU first.
  ./target/release/bench_native > /dev/null
  : > "$OUT/raw_bench_pass${PASS:-1}.tsv"
  for e in $ENGINES; do
    echo "== bench_$e" | tee -a "$OUT/raw_bench_pass${PASS:-1}.tsv"
    ./target/release/bench_$e 2>&1 | tee -a "$OUT/raw_bench_pass${PASS:-1}.tsv"
  done
fi

# Process wall time: boot binaries (create engine, eval trivial script, print, exit).
cmds=()
for e in $ENGINES; do cmds+=("./target/release/boot_$e"); done
cmds+=("./target/release/boot_wasmtime_precompiled $OUT/bench.cwasm")
hyperfine -N --warmup 5 --runs "${HF_RUNS:-50}" --export-json "$OUT/hyperfine_pass${PASS:-1}.json" --export-markdown "$OUT/hyperfine_pass${PASS:-1}.md" "${cmds[@]}" 2>&1 | tee "$OUT/hyperfine_pass${PASS:-1}.txt"

# Binary sizes (release profile has strip = true).
: > "$OUT/sizes.tsv"
for b in target/release/boot_* target/release/bench_*; do
  [[ -x $b && ! -d $b ]] || continue
  printf "%s\t%s\n" "$(basename "$b")" "$(wc -c < "$b" | tr -d " ")" >> "$OUT/sizes.tsv"
done
if command -v otool > /dev/null; then otool -L target/release/boot_pyo3 > "$OUT/pyo3_otool.txt"; fi
