# `add` is a host (Rust) function injected into the module globals.
def fib(n):
    return n if n < 2 else fib(n - 1) + fib(n - 2)

def loop_sum(n):
    s = 0
    for i in range(n):
        s += i
    return s

def str_build(n):
    parts = []
    for i in range(n):
        parts.append(str(i))
    return len("".join(parts))

def host_loop(n):
    acc = 0
    for i in range(n):
        acc = add(acc, 1)
    return acc

def inc(x):
    return x + 1
