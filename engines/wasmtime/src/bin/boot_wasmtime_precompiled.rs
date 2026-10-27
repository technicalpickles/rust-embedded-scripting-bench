// Usage: boot_wasmtime_precompiled <path to .cwasm written by `bench_wasmtime <path>`>
use wasmtime::{Engine, Linker, Module, Store};
fn main() {
    let path = std::env::args().nth(1).expect("cwasm path");
    let engine = Engine::default();
    let module = unsafe { Module::deserialize_file(&engine, path).unwrap() };
    let mut l = Linker::new(&engine);
    l.func_wrap("env", "add", |a: i64, b: i64| a + b).unwrap();
    let mut store = Store::new(&engine, ());
    let inst = l.instantiate(&mut store, &module).unwrap();
    let v = inst.get_typed_func::<(), i64>(&mut store, "answer").unwrap().call(&mut store, ()).unwrap();
    println!("{v}");
}
