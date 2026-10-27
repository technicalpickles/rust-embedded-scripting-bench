// Included (via #[path]) by each mlua_* crate; the crate's Cargo features pick the Lua flavor.
use harness::{boot, suite, Cfg, W};
use mlua::{Function, Lua};

pub fn run(engine: &str) {
    let cfg = Cfg::from_env();
    boot(engine, "new+eval", &cfg, 2, || {
        let lua = Lua::new();
        lua.load("return 1+1").eval::<i64>().unwrap()
    });

    let lua = Lua::new();
    let add = lua.create_function(|_, (a, b): (i64, i64)| Ok(a + b)).unwrap();
    lua.globals().set("add", add).unwrap();
    lua.load(include_str!("../scripts/bench.lua")).exec().unwrap();
    let g = lua.globals();
    let fib: Function = g.get("fib").unwrap();
    let loop_sum: Function = g.get("loop_sum").unwrap();
    let str_build: Function = g.get("str_build").unwrap();
    let host_loop: Function = g.get("host_loop").unwrap();
    let inc: Function = g.get("inc").unwrap();
    suite(
        engine,
        &cfg,
        |w, n| {
            let f = match w {
                W::Fib => &fib,
                W::LoopSum => &loop_sum,
                W::StrBuild => &str_build,
                W::HostLoop => &host_loop,
            };
            Some(f.call::<i64>(n).unwrap())
        },
        Some(|i: i64| inc.call::<i64>(i).unwrap()),
    );
}

pub fn boot_main() {
    let lua = Lua::new();
    let v: i64 = lua.load("return 1+1").eval().unwrap();
    println!("{v}");
}
