fn main() {
    let rt = rquickjs::Runtime::new().unwrap();
    let ctx = rquickjs::Context::full(&rt).unwrap();
    let v: i32 = ctx.with(|ctx| ctx.eval("1+1")).unwrap();
    println!("{v}");
}
