-- Shared by Lua 5.4, LuaJIT, Luau. `add` is a host (Rust) function registered as a global.
function fib(n)
  if n < 2 then return n end
  return fib(n - 1) + fib(n - 2)
end

function loop_sum(n)
  local s = 0
  for i = 0, n - 1 do s = s + i end
  return s
end

function str_build(n)
  local t = {}
  for i = 0, n - 1 do t[i + 1] = tostring(i) end
  return #table.concat(t)
end

function host_loop(n)
  local acc = 0
  for i = 1, n do acc = add(acc, 1) end
  return acc
end

function inc(x) return x + 1 end
