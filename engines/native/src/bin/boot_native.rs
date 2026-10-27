fn main() {
    let v = std::hint::black_box(1i64) + 1;
    println!("{v}");
}
