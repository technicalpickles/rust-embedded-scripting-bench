use pyo3::prelude::*;
fn main() {
    let v: i64 = Python::attach(|py| py.eval(c"1+1", None, None).unwrap().extract().unwrap());
    println!("{v}");
}
