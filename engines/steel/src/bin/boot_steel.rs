use steel::steel_vm::engine::Engine;
fn main() {
    let mut e = Engine::new();
    let v = e.run("(+ 1 1)").unwrap();
    println!("{:?}", v.last().unwrap());
}
