use boa_engine::{js_string, Context, JsResult, JsValue, NativeFunction, Source};
use harness::{boot, suite, Cfg, W};

fn add(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let a = args[0].to_number(ctx)?;
    let b = args[1].to_number(ctx)?;
    Ok(JsValue::from(a + b))
}

fn main() {
    let cfg = Cfg::from_env();
    boot("boa", "context+eval", &cfg, 2, || {
        let mut ctx = Context::default();
        let v = ctx.eval(Source::from_bytes("1+1")).unwrap();
        v.to_number(&mut ctx).unwrap() as i64
    });

    let mut ctx = Context::default();
    ctx.register_global_callable(js_string!("add"), 2, NativeFunction::from_fn_ptr(add)).unwrap();
    ctx.eval(Source::from_bytes(include_str!("../../../../scripts/bench.js"))).unwrap();
    let get = |ctx: &mut Context, name: &str| {
        let g = ctx.global_object();
        g.get(js_string!(name), ctx).unwrap().as_callable().unwrap().clone()
    };
    let fns: Vec<_> = ["fib", "loop_sum", "str_build", "host_loop", "inc"]
        .iter()
        .map(|n| get(&mut ctx, n))
        .collect();
    let ctx = std::cell::RefCell::new(ctx);
    let call = |idx: usize, n: i64| -> i64 {
        let mut ctx = ctx.borrow_mut();
        let r = fns[idx].call(&JsValue::undefined(), &[JsValue::from(n as f64)], &mut ctx).unwrap();
        r.to_number(&mut ctx).unwrap() as i64
    };
    suite(
        "boa",
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
