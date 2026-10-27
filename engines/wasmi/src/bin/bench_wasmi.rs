use harness::{boot, suite, Cfg, W};
use wasmi::{Engine, Linker, Module, Store};

const WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/bench.wasm"));

fn linker(engine: &Engine) -> Linker<()> {
    let mut l = Linker::new(engine);
    l.func_wrap("env", "add", |a: i64, b: i64| a + b).unwrap();
    l
}
fn answer(engine: &Engine, module: &Module) -> i64 {
    let mut store = Store::new(engine, ());
    let inst = linker(engine).instantiate_and_start(&mut store, module).unwrap();
    inst.get_typed_func::<(), i64>(&store, "answer").unwrap().call(&mut store, ()).unwrap()
}

fn main() {
    let cfg = Cfg::from_env();
    boot("wasmi", "engine+compile+instantiate+call", &cfg, 42, || {
        let engine = Engine::default();
        let module = Module::new(&engine, WASM).unwrap();
        answer(&engine, &module)
    });
    let engine0 = Engine::default();
    let module0 = Module::new(&engine0, WASM).unwrap();
    boot("wasmi", "warm_module:instantiate+call", &cfg, 42, || answer(&engine0, &module0));

    let mut store = Store::new(&engine0, ());
    let inst = linker(&engine0).instantiate_and_start(&mut store, &module0).unwrap();
    let f = |name| inst.get_typed_func::<i64, i64>(&store, name).unwrap();
    let fns = [f("fib"), f("loop_sum"), f("host_loop"), f("inc")];
    let store = std::cell::RefCell::new(store);
    suite(
        "wasmi",
        &cfg,
        |w, n| {
            let idx = match w {
                W::Fib => 0,
                W::LoopSum => 1,
                W::StrBuild => return None,
                W::HostLoop => 2,
            };
            Some(fns[idx].call(&mut *store.borrow_mut(), n).unwrap())
        },
        Some(|i: i64| fns[3].call(&mut *store.borrow_mut(), i).unwrap()),
    );
}
