use wasmi::{Engine, Linker, Module, Store};
const WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/bench.wasm"));
fn main() {
    let engine = Engine::default();
    let module = Module::new(&engine, WASM).unwrap();
    let mut l = Linker::new(&engine);
    l.func_wrap("env", "add", |a: i64, b: i64| a + b).unwrap();
    let mut store = Store::new(&engine, ());
    let inst = l.instantiate_and_start(&mut store, &module).unwrap();
    let v = inst.get_typed_func::<(), i64>(&store, "answer").unwrap().call(&mut store, ()).unwrap();
    println!("{v}");
}
