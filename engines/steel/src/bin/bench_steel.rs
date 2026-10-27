use harness::{boot, suite, Cfg, W};
use steel::rvals::SteelVal;
use steel::steel_vm::engine::Engine;
use steel::steel_vm::register_fn::RegisterFn;

fn as_i64(v: SteelVal) -> i64 {
    match v {
        SteelVal::IntV(i) => i as i64,
        other => panic!("unexpected steel value {other:?}"),
    }
}

fn main() {
    let cfg = Cfg::from_env();
    boot("steel", "new+run", &cfg, 2, || {
        let mut e = Engine::new();
        as_i64(e.run("(+ 1 1)").unwrap().pop().unwrap())
    });
    boot("steel", "new_base+run", &cfg, 2, || {
        let mut e = Engine::new_base();
        as_i64(e.run("(+ 1 1)").unwrap().pop().unwrap())
    });

    let mut engine = Engine::new();
    engine.register_fn("add", |a: isize, b: isize| a + b);
    engine.run(include_str!("../../../../scripts/bench.scm")).unwrap();
    let engine = std::cell::RefCell::new(engine);
    let call = |name: &str, n: i64| -> i64 {
        as_i64(engine.borrow_mut().call_function_by_name_with_args(name, vec![SteelVal::IntV(n as isize)]).unwrap())
    };
    suite(
        "steel",
        &cfg,
        |w, n| {
            Some(call(
                match w {
                    W::Fib => "fib",
                    W::LoopSum => "loop-sum",
                    W::StrBuild => "str-build",
                    W::HostLoop => "host-loop",
                },
                n,
            ))
        },
        Some(|i: i64| call("inc", i)),
    );
}
