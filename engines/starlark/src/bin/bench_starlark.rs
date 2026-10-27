use harness::{boot, suite, Cfg, W};
use starlark::environment::{Globals, GlobalsBuilder, Module};
use starlark::eval::Evaluator;
use starlark::starlark_module;
use starlark::syntax::{AstModule, Dialect};
use starlark::values::UnpackValue;

#[starlark_module]
fn host(builder: &mut GlobalsBuilder) {
    fn add(a: i64, b: i64) -> anyhow::Result<i64> {
        Ok(a + b)
    }
}

fn main() {
    let cfg = Cfg::from_env();
    boot("starlark", "globals+parse+eval", &cfg, 2, || {
        let globals = Globals::standard();
        let ast = AstModule::parse("t.star", "1+1".to_owned(), &Dialect::Standard).unwrap();
        Module::with_temp_heap(|module| {
            let mut eval = Evaluator::new(&module);
            let v = eval.eval_module(ast, &globals).unwrap();
            i64::unpack_value(v).unwrap().unwrap()
        })
    });

    let globals = GlobalsBuilder::standard().with(host).build();
    let ast = AstModule::parse("bench.star", include_str!("../../../../scripts/bench.star").to_owned(), &Dialect::Standard).unwrap();
    Module::with_temp_heap(|module| {
        let mut eval = Evaluator::new(&module);
        eval.eval_module(ast, &globals).unwrap();
        let get = |n: &str| module.get(n).unwrap();
        let fns = [get("fib"), get("loop_sum"), get("str_build"), get("host_loop"), get("inc")];
        let eval = std::cell::RefCell::new(eval);
        let call = |idx: usize, n: i64| -> i64 {
            let mut e = eval.borrow_mut();
            let arg = module.heap().alloc(n);
            let v = e.eval_function(fns[idx], &[arg], &[]).unwrap();
            i64::unpack_value(v).unwrap().unwrap()
        };
        suite(
            "starlark",
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
    });
}
