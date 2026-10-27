// `add` is a host (Rust) function registered on the Engine.
fn fib(n) { if n < 2 { n } else { fib(n - 1) + fib(n - 2) } }
fn loop_sum(n) { let s = 0; for i in 0..n { s += i; } s }
fn str_build(n) { let s = ""; for i in 0..n { s += i.to_string(); } s.len() }
fn host_loop(n) { let acc = 0; for i in 0..n { acc = add(acc, 1); } acc }
fn inc(x) { x + 1 }
