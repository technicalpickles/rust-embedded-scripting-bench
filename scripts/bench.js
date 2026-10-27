// Shared by rquickjs and boa. `add` is a host (Rust) function registered as a global.
function fib(n) { return n < 2 ? n : fib(n - 1) + fib(n - 2); }
function loop_sum(n) { let s = 0; for (let i = 0; i < n; i++) { s += i; } return s; }
function str_build(n) { const parts = []; for (let i = 0; i < n; i++) { parts.push(String(i)); } return parts.join('').length; }
function host_loop(n) { let acc = 0; for (let i = 0; i < n; i++) { acc = add(acc, 1); } return acc; }
function inc(x) { return x + 1; }
