use starlark::environment::{Globals, Module};
use starlark::eval::Evaluator;
use starlark::syntax::{AstModule, Dialect};
fn main() {
    let globals = Globals::standard();
    let ast = AstModule::parse("t.star", "1+1".to_owned(), &Dialect::Standard).unwrap();
    Module::with_temp_heap(|module| {
        let mut eval = Evaluator::new(&module);
        let v = eval.eval_module(ast, &globals).unwrap();
        println!("{}", v);
    });
}
