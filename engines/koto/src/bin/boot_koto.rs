use koto::prelude::*;
fn main() {
    let mut koto = Koto::default();
    let v = koto.compile_and_run("1 + 1").unwrap();
    println!("{}", koto.value_to_string(v).unwrap());
}
