(module
  (import "env" "add" (func $add (param i64 i64) (result i64)))
  (func $fib (export "fib") (param $n i64) (result i64)
    (if (result i64) (i64.lt_s (local.get $n) (i64.const 2))
      (then (local.get $n))
      (else (i64.add
        (call $fib (i64.sub (local.get $n) (i64.const 1)))
        (call $fib (i64.sub (local.get $n) (i64.const 2)))))))
  (func (export "loop_sum") (param $n i64) (result i64) (local $i i64) (local $s i64)
    (block $done
      (loop $l
        (br_if $done (i64.ge_s (local.get $i) (local.get $n)))
        (local.set $s (i64.add (local.get $s) (local.get $i)))
        (local.set $i (i64.add (local.get $i) (i64.const 1)))
        (br $l)))
    (local.get $s))
  (func (export "host_loop") (param $n i64) (result i64) (local $i i64) (local $acc i64)
    (block $done
      (loop $l
        (br_if $done (i64.ge_s (local.get $i) (local.get $n)))
        (local.set $acc (call $add (local.get $acc) (i64.const 1)))
        (local.set $i (i64.add (local.get $i) (i64.const 1)))
        (br $l)))
    (local.get $acc))
  (func (export "inc") (param $x i64) (result i64) (i64.add (local.get $x) (i64.const 1)))
  (func (export "answer") (result i64) (i64.const 42)))
