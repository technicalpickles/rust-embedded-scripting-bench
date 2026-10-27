fn main() {
    let engine = rhai::Engine::new();
    let v: i64 = engine.eval("1+1").unwrap();
    println!("{v}");
}
