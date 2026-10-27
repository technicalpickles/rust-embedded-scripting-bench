use rune::{Context, Diagnostics, Source, Sources, Vm};
use std::sync::Arc;
fn main() {
    let context = Context::with_default_modules().unwrap();
    let runtime = Arc::new(context.runtime().unwrap());
    let mut sources = Sources::new();
    sources.insert(Source::memory("pub fn main() { 1 + 1 }").unwrap()).unwrap();
    let mut diagnostics = Diagnostics::new();
    let unit = rune::prepare(&mut sources).with_context(&context).with_diagnostics(&mut diagnostics).build().unwrap();
    let mut vm = Vm::new(runtime, Arc::new(unit));
    let v: i64 = rune::from_value(vm.call(["main"], ()).unwrap()).unwrap();
    println!("{v}");
}
