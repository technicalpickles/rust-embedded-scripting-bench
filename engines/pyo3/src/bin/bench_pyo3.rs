use harness::{once, suite, Cfg, W};
use pyo3::prelude::*;
use std::ffi::CString;
use std::time::Instant;

#[pyfunction]
fn add(a: i64, b: i64) -> i64 {
    a + b
}

fn main() {
    let cfg = Cfg::from_env();
    // CPython can only be initialized once per process, so boot is a single sample here;
    // the process-level (hyperfine) number is the meaningful one.
    let t = Instant::now();
    let v: i64 = Python::attach(|py| py.eval(c"1+1", None, None).unwrap().extract().unwrap());
    once("pyo3", "boot:first_initialize+eval", t.elapsed(), "single sample (CPython init is once per process)");
    assert_eq!(v, 2);
    let mut samples = Vec::new();
    for _ in 0..cfg.boot_iters {
        let t = Instant::now();
        let v: i64 = Python::attach(|py| py.eval(c"1+1", None, None).unwrap().extract().unwrap());
        samples.push(t.elapsed().as_secs_f64() * 1e6);
        assert_eq!(v, 2);
    }
    harness::emit("pyo3", "boot:warm_interp_attach+eval", samples.len() as i64, &mut samples, "us", "interpreter already initialized; not a cold boot");

    Python::attach(|py| {
        let src = CString::new(include_str!("../../../../scripts/bench.py")).unwrap();
        let m = PyModule::from_code(py, &src, c"bench.py", c"bench").unwrap();
        m.add_function(wrap_pyfunction!(add, &m).unwrap()).unwrap();
        let get = |n: &str| m.getattr(n).unwrap();
        let fns = [get("fib"), get("loop_sum"), get("str_build"), get("host_loop"), get("inc")];
        let call = |idx: usize, n: i64| -> i64 { fns[idx].call1((n,)).unwrap().extract().unwrap() };
        suite(
            "pyo3",
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
