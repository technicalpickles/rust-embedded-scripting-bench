// Assemble scripts/bench.wat -> $OUT_DIR/bench.wasm so binaries can include_bytes! it
// (keeps WAT parsing out of every timed region).
fn main() {
    let src = "../../scripts/bench.wat";
    println!("cargo:rerun-if-changed={src}");
    let wasm = wat::parse_file(src).unwrap();
    let out = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("bench.wasm");
    std::fs::write(out, wasm).unwrap();
}
