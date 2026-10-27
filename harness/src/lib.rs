//! Shared timing harness. Every engine binary prints lines of the form
//! RESULT\t<engine>\t<metric>\t<n>\t<median>\t<p95>\t<min>\t<unit>\t<note>
use std::time::{Duration, Instant};

pub struct Cfg {
    pub fib_n: i64,
    pub loop_n: i64,
    pub str_n: i64,
    pub host_n: i64,
    pub call_n: i64,
    pub boot_iters: usize,
    pub boot_budget: Duration,
    pub runs: usize,
}

fn env_i64(k: &str, d: i64) -> i64 {
    std::env::var(k).ok().and_then(|v| v.parse().ok()).unwrap_or(d)
}

impl Cfg {
    pub fn from_env() -> Self {
        Cfg {
            fib_n: env_i64("FIB_N", 30),
            loop_n: env_i64("LOOP_N", 10_000_000),
            str_n: env_i64("STR_N", 100_000),
            host_n: env_i64("HOST_N", 1_000_000),
            call_n: env_i64("CALL_N", 100_000),
            boot_iters: env_i64("BOOT_ITERS", 1000) as usize,
            boot_budget: Duration::from_secs(env_i64("BOOT_BUDGET_S", 10) as u64),
            runs: env_i64("RUNS", 5) as usize,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum W {
    Fib,
    LoopSum,
    StrBuild,
    HostLoop,
}

pub fn expected(w: W, n: i64) -> i64 {
    match w {
        W::Fib => {
            let (mut a, mut b) = (0i64, 1i64);
            for _ in 0..n {
                let t = a + b;
                a = b;
                b = t;
            }
            a
        }
        W::LoopSum => n * (n - 1) / 2,
        W::StrBuild => (0..n).map(|i| i.to_string().len() as i64).sum(),
        W::HostLoop => n,
    }
}

fn pct(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx]
}

pub fn emit(engine: &str, metric: &str, n: i64, samples: &mut Vec<f64>, unit: &str, note: &str) {
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!(
        "RESULT\t{engine}\t{metric}\t{n}\t{:.3}\t{:.3}\t{:.3}\t{unit}\t{note}",
        pct(samples, 0.5),
        pct(samples, 0.95),
        samples[0]
    );
}

/// Boot benchmark: `f` must create a fresh engine, evaluate the trivial script and
/// return its integer result (checked against `expect`). Timed per iteration in µs.
/// Runs `cfg.boot_iters` iterations or until the time budget is exhausted (min 20).
pub fn boot<F: FnMut() -> i64>(engine: &str, variant: &str, cfg: &Cfg, expect: i64, mut f: F) {
    // one warmup (page-in code, lazy statics)
    assert_eq!(f(), expect, "{engine} boot result");
    let mut samples = Vec::with_capacity(cfg.boot_iters);
    let start = Instant::now();
    for i in 0..cfg.boot_iters {
        let t = Instant::now();
        let r = f();
        let dt = t.elapsed();
        std::hint::black_box(r);
        samples.push(dt.as_secs_f64() * 1e6);
        if i >= 20 && start.elapsed() > cfg.boot_budget {
            break;
        }
    }
    let n = samples.len() as i64;
    emit(engine, &format!("boot:{variant}"), n, &mut samples, "us", "n=iterations");
}

/// Single-shot timing (for things that can only happen once per process).
pub fn once(engine: &str, metric: &str, dt: Duration, note: &str) {
    let mut s = vec![dt.as_secs_f64() * 1e6];
    emit(engine, metric, 1, &mut s, "us", note);
}

fn wname(w: W) -> &'static str {
    match w {
        W::Fib => "fib",
        W::LoopSum => "loop_sum",
        W::StrBuild => "str_build",
        W::HostLoop => "host_call",
    }
}

/// Throughput suite. `call(w, n)` runs workload `w` with size `n` inside the warm engine
/// and returns its result, or None if unsupported. `inc(i)` calls the script function
/// `inc` from Rust once (None => unsupported).
pub fn suite<C, I>(engine: &str, cfg: &Cfg, mut call: C, mut inc: Option<I>)
where
    C: FnMut(W, i64) -> Option<i64>,
    I: FnMut(i64) -> i64,
{
    for (w, n) in [
        (W::Fib, cfg.fib_n),
        (W::LoopSum, cfg.loop_n),
        (W::StrBuild, cfg.str_n),
        (W::HostLoop, cfg.host_n),
    ] {
        // warmup + correctness
        match call(w, n) {
            None => {
                println!("SKIP\t{engine}\t{}\tunsupported", wname(w));
                continue;
            }
            Some(r) => assert_eq!(r, expected(w, n), "{engine} {w:?}({n})"),
        }
        let mut samples = Vec::new();
        for _ in 0..cfg.runs {
            let t = Instant::now();
            let r = call(w, n);
            samples.push(t.elapsed().as_secs_f64() * 1e3);
            std::hint::black_box(r);
        }
        emit(engine, wname(w), n, &mut samples, "ms", "");
    }
    if let Some(inc) = inc.as_mut() {
        let n = cfg.call_n;
        let run = |inc: &mut I| {
            let mut s = 0i64;
            for i in 0..n {
                s += inc(i);
            }
            s
        };
        assert_eq!(run(inc), n * (n + 1) / 2, "{engine} inc");
        let mut samples = Vec::new();
        for _ in 0..cfg.runs {
            let t = Instant::now();
            let r = run(inc);
            samples.push(t.elapsed().as_secs_f64() * 1e3);
            std::hint::black_box(r);
        }
        emit(engine, "call_from_rust", n, &mut samples, "ms", "");
    } else {
        println!("SKIP\t{engine}\tcall_from_rust\tunsupported");
    }
}
