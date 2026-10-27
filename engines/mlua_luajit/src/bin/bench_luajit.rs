#[path = "../../../../shared/mlua_bench.rs"]
mod mlua_bench;
fn main() { mlua_bench::run("luajit"); }
