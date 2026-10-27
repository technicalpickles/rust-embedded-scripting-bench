use harness::{boot, suite, Cfg, W};
use koto::prelude::*;

fn as_i64(v: KValue) -> i64 {
    match v {
        KValue::Number(n) => i64::from(n),
        other => panic!("unexpected koto value: {}", other.type_as_string()),
    }
}

fn main() {
    let cfg = Cfg::from_env();
    boot("koto", "new+compile_and_run", &cfg, 2, || {
        let mut koto = Koto::default();
        as_i64(koto.compile_and_run("1 + 1").unwrap())
    });

    let mut koto = Koto::default();
    koto.prelude().add_fn("add", |ctx| match ctx.args() {
        [KValue::Number(a), KValue::Number(b)] => Ok((*a + *b).into()),
        unexpected => unexpected_args("|Number, Number|", unexpected),
    });
    koto.compile_and_run(include_str!("../../../../scripts/bench.koto")).unwrap();
    let koto = std::cell::RefCell::new(koto);
    let call = |name: &str, n: i64| -> i64 {
        as_i64(koto.borrow_mut().call_exported_function(name, &[n.into()]).unwrap())
    };
    suite(
        "koto",
        &cfg,
        |w, n| {
            Some(call(
                match w {
                    W::Fib => "fib",
                    W::LoopSum => "loop_sum",
                    W::StrBuild => "str_build",
                    W::HostLoop => "host_loop",
                },
                n,
            ))
        },
        Some(|i: i64| call("inc", i)),
    );
}
