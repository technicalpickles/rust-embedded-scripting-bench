use harness::{boot, suite, Cfg, W};
use rune::{Context, Diagnostics, Module, Source, Sources, Vm};
use std::sync::Arc;

pub fn make_vm(src: &str, with_add: bool) -> Vm {
    let mut context = Context::with_default_modules().unwrap();
    if with_add {
        let mut m = Module::new();
        m.function("add", |a: i64, b: i64| a + b).build().unwrap();
        context.install(m).unwrap();
    }
    let runtime = Arc::new(context.runtime().unwrap());
    let mut sources = Sources::new();
    sources.insert(Source::memory(src).unwrap()).unwrap();
    let mut diagnostics = Diagnostics::new();
    let unit = rune::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .build();
    if !diagnostics.is_empty() {
        let mut w = rune::termcolor::StandardStream::stderr(rune::termcolor::ColorChoice::Never);
        diagnostics.emit(&mut w, &sources).unwrap();
    }
    Vm::new(runtime, Arc::new(unit.unwrap()))
}

fn main() {
    let cfg = Cfg::from_env();
    boot("rune", "context+compile+call", &cfg, 2, || {
        let mut vm = make_vm("pub fn main() { 1 + 1 }", false);
        rune::from_value::<i64>(vm.call(["main"], ()).unwrap()).unwrap()
    });

    // Variant: Context + RuntimeContext built once, per-iteration compile + VM + call.
    let context = Context::with_default_modules().unwrap();
    let runtime = Arc::new(context.runtime().unwrap());
    boot("rune", "warm_context:compile+call", &cfg, 2, || {
        let mut sources = Sources::new();
        sources.insert(Source::memory("pub fn main() { 1 + 1 }").unwrap()).unwrap();
        let unit = rune::prepare(&mut sources).with_context(&context).build().unwrap();
        let mut vm = Vm::new(runtime.clone(), Arc::new(unit));
        rune::from_value::<i64>(vm.call(["main"], ()).unwrap()).unwrap()
    });
    boot("rune", "context_only", &cfg, 0, || {
        std::hint::black_box(Context::with_default_modules().unwrap());
        0
    });

    let vm = std::cell::RefCell::new(make_vm(include_str!("../../../../scripts/bench.rn"), true));
    let call = |name: &str, n: i64| -> i64 {
        let out = vm.borrow_mut().call([name], (n,)).unwrap();
        rune::from_value::<i64>(out).unwrap()
    };
    suite(
        "rune",
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
