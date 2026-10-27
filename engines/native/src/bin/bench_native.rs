use harness::{suite, Cfg, W};
use std::fmt::Write;
use std::hint::black_box;

fn fib(n: i64) -> i64 {
    if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}
fn loop_sum(n: i64) -> i64 {
    let mut s = 0i64;
    for i in 0..n {
        s += black_box(i); // black_box stops LLVM from closed-forming / vectorizing the sum
    }
    s
}
fn str_build(n: i64) -> i64 {
    let mut s = String::new();
    for i in 0..n {
        write!(s, "{i}").unwrap();
    }
    s.len() as i64
}
#[inline(never)]
fn add(a: i64, b: i64) -> i64 { a + b }
fn host_loop(n: i64) -> i64 {
    let f: fn(i64, i64) -> i64 = black_box(add); // opaque fn pointer = un-inlinable "host" call
    let mut acc = 0;
    for _ in 0..n {
        acc = f(acc, 1);
    }
    acc
}
#[inline(never)]
fn inc(x: i64) -> i64 { x + 1 }

fn main() {
    let cfg = Cfg::from_env();
    let incf: fn(i64) -> i64 = black_box(inc);
    suite(
        "native",
        &cfg,
        |w, n| {
            let n = black_box(n);
            Some(match w {
                W::Fib => fib(n),
                W::LoopSum => loop_sum(n),
                W::StrBuild => str_build(n),
                W::HostLoop => host_loop(n),
            })
        },
        Some(|i: i64| incf(black_box(i))),
    );
}
