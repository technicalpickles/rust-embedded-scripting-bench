use boa_engine::{Context, Source};
fn main() {
    let mut ctx = Context::default();
    let v = ctx.eval(Source::from_bytes("1+1")).unwrap();
    println!("{}", v.to_number(&mut ctx).unwrap());
}
