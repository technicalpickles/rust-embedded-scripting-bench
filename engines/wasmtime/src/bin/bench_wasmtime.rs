use harness::{boot, suite, Cfg, W};
use wasmtime::{Engine, Instance, Linker, Module, Store};

const WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/bench.wasm"));

fn linker(engine: &Engine) -> Linker<()> {
    let mut l = Linker::new(engine);
    l.func_wrap("env", "add", |a: i64, b: i64| a + b).unwrap();
    l
}
fn answer(engine: &Engine, module: &Module) -> i64 {
    let mut store = Store::new(engine, ());
    let inst = linker(engine).instantiate(&mut store, module).unwrap();
    inst.get_typed_func::<(), i64>(&mut store, "answer").unwrap().call(&mut store, ()).unwrap()
}

fn main() {
    let cfg = Cfg::from_env();
    // Optional: write the precompiled artifact for boot_wasmtime_precompiled.
    if let Some(path) = std::env::args().nth(1) {
        let engine = Engine::default();
        std::fs::write(&path, engine.precompile_module(WASM).unwrap()).unwrap();
        eprintln!("wrote {path}");
        return;
    }
    let engine0 = Engine::default();
    let cwasm = engine0.precompile_module(WASM).unwrap();
    println!("INFO\twasmtime\twasm_bytes={}\tcwasm_bytes={}", WASM.len(), cwasm.len());

    boot("wasmtime", "engine+compile+instantiate+call", &cfg, 42, || {
        let engine = Engine::default();
        let module = Module::new(&engine, WASM).unwrap();
        answer(&engine, &module)
    });
    boot("wasmtime", "engine+deserialize+instantiate+call", &cfg, 42, || {
        let engine = Engine::default();
        let module = unsafe { Module::deserialize(&engine, &cwasm).unwrap() };
        answer(&engine, &module)
    });
    boot("wasmtime", "engine_new_only", &cfg, 0, || {
        std::hint::black_box(Engine::default());
        0
    });
    boot("wasmtime", "warm_engine:compile+instantiate+call", &cfg, 42, || {
        let module = Module::new(&engine0, WASM).unwrap();
        answer(&engine0, &module)
    });
    boot("wasmtime", "warm_engine:deserialize+instantiate+call", &cfg, 42, || {
        let module = unsafe { Module::deserialize(&engine0, &cwasm).unwrap() };
        answer(&engine0, &module)
    });
    let module0 = Module::new(&engine0, WASM).unwrap();
    let linker0 = linker(&engine0);
    let pre = linker0.instantiate_pre(&module0).unwrap();
    boot("wasmtime", "warm_module:instantiate+call", &cfg, 42, || answer(&engine0, &module0));
    boot("wasmtime", "warm_instance_pre:instantiate+call", &cfg, 42, || {
        let mut store = Store::new(&engine0, ());
        let inst: Instance = pre.instantiate(&mut store).unwrap();
        inst.get_typed_func::<(), i64>(&mut store, "answer").unwrap().call(&mut store, ()).unwrap()
    });

    let mut store = Store::new(&engine0, ());
    let inst = linker0.instantiate(&mut store, &module0).unwrap();
    let f = |s: &mut Store<()>, name| inst.get_typed_func::<i64, i64>(s, name).unwrap();
    let fns = [f(&mut store, "fib"), f(&mut store, "loop_sum"), f(&mut store, "host_loop"), f(&mut store, "inc")];
    let store = std::cell::RefCell::new(store);
    suite(
        "wasmtime",
        &cfg,
        |w, n| {
            let idx = match w {
                W::Fib => 0,
                W::LoopSum => 1,
                W::StrBuild => return None, // no string type in core wasm without a guest runtime
                W::HostLoop => 2,
            };
            Some(fns[idx].call(&mut *store.borrow_mut(), n).unwrap())
        },
        Some(|i: i64| fns[3].call(&mut *store.borrow_mut(), i).unwrap()),
    );
}
