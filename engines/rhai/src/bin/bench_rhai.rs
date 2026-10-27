use harness::{boot, suite, Cfg, W};
use rhai::{Engine, Scope};

fn main() {
    let cfg = Cfg::from_env();
    boot("rhai", "new+eval", &cfg, 2, || Engine::new().eval::<i64>("1+1").unwrap());
    boot("rhai", "new_raw+eval", &cfg, 2, || Engine::new_raw().eval::<i64>("1+1").unwrap());

    let mut engine = Engine::new();
    engine.register_fn("add", |a: i64, b: i64| a + b);
    let ast = engine.compile(include_str!("../../../../scripts/bench.rhai")).unwrap();
    let mut scope = Scope::new();
    let name = |w| match w {
        W::Fib => "fib",
        W::LoopSum => "loop_sum",
        W::StrBuild => "str_build",
        W::HostLoop => "host_loop",
    };
    let engine = &engine;
    let ast = &ast;
    let mut scope2 = Scope::new();
    suite(
        "rhai",
        &cfg,
        |w, n| Some(engine.call_fn::<i64>(&mut scope, ast, name(w), (n,)).unwrap()),
        Some(|i: i64| engine.call_fn::<i64>(&mut scope2, ast, "inc", (i,)).unwrap()),
    );
}
