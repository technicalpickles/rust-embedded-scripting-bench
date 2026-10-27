use harness::{boot, suite, Cfg, W};
use rquickjs::{Context, Function, Persistent, Runtime};

fn main() {
    let cfg = Cfg::from_env();
    boot("rquickjs", "runtime+context_full+eval", &cfg, 2, || {
        let rt = Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| ctx.eval::<i32, _>("1+1")).unwrap() as i64
    });

    let rt = Runtime::new().unwrap();
    let ctx = Context::full(&rt).unwrap();
    let fns: Vec<Persistent<Function<'static>>> = ctx.with(|ctx| {
        let add = Function::new(ctx.clone(), |a: f64, b: f64| a + b).unwrap();
        ctx.globals().set("add", add).unwrap();
        ctx.eval::<(), _>(include_str!("../../../../scripts/bench.js")).unwrap();
        ["fib", "loop_sum", "str_build", "host_loop", "inc"]
            .iter()
            .map(|n| Persistent::save(&ctx, ctx.globals().get::<_, Function>(*n).unwrap()))
            .collect()
    });
    let call = |idx: usize, n: i64| -> i64 {
        ctx.with(|ctx| {
            let f = fns[idx].clone().restore(&ctx).unwrap();
            f.call::<_, f64>((n as f64,)).unwrap() as i64
        })
    };
    suite(
        "rquickjs",
        &cfg,
        |w, n| {
            Some(call(
                match w {
                    W::Fib => 0,
                    W::LoopSum => 1,
                    W::StrBuild => 2,
                    W::HostLoop => 3,
                },
                n,
            ))
        },
        Some(|i: i64| call(4, i)),
    );
}
